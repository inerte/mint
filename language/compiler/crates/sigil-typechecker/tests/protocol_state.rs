//! Protocol state enforcement tests.
//!
//! Verifies that the compiler correctly enforces protocol state machine contracts:
//! - State violations at call sites are rejected
//! - Valid protocol usage compiles
//! - Error codes are correct
//! - State propagates through match arms correctly

use sigil_lexer::tokenize;
use sigil_parser::parse;
use sigil_typechecker::types::{TFunction, TPrimitive, TRecord};
use sigil_typechecker::{type_check, InferenceType, TypeCheckOptions, TypeInfo};
use std::collections::HashMap;

fn typecheck(
    source: &str,
) -> Result<sigil_typechecker::TypeCheckResult, sigil_typechecker::TypeError> {
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();
    type_check(&program, source, None)
}

fn typecheck_with_options(
    source: &str,
    options: TypeCheckOptions,
) -> Result<sigil_typechecker::TypeCheckResult, sigil_typechecker::TypeError> {
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, "test.sigil").unwrap();
    type_check(&program, source, Some(options))
}

const TICKET_PROTOCOL: &str = concat!(
    "t Ticket={id:String}\n",
    "protocol Ticket\n",
    "  Open → Closed via resolve\n",
    "  Open → Open via addNote\n",
    "  initial = Open\n",
    "  terminal = Closed\n",
    "λaddNote(note:String,ticket:Ticket)=>Ticket\n",
    "requires ticket.state=Open\n",
    "ensures result.state=Open\n",
    "={id:ticket.id}\n",
    "λresolve(ticket:Ticket)=>Bool\n",
    "requires ticket.state=Open\n",
    "ensures ticket.state=Closed\n",
    "=true\n",
);

// ============================================================================
// Valid protocol usage
// ============================================================================

#[test]
fn valid_single_call_open_state() {
    let source = format!(
        "{}λmain()=>Bool={{l ticket=({{id:\"T-1\"}}:Ticket);resolve(ticket)}}",
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    assert!(r.is_ok(), "Single call in Open state should succeed: {r:?}");
}

#[test]
fn valid_protocol_declaration_compiles() {
    let source = format!("{}λmain()=>Bool=true", TICKET_PROTOCOL);
    let r = typecheck(&source);
    assert!(r.is_ok(), "Protocol declaration should compile: {r:?}");
}

#[test]
fn valid_state_contracts_on_functions() {
    let source = concat!(
        "t Conn={id:String}\n",
        "protocol Conn\n",
        "  Open → Closed via disconnect\n",
        "  Open → Open via ping\n",
        "  initial = Open\n",
        "  terminal = Closed\n",
        "λping(conn:Conn)=>Bool\n",
        "requires conn.state=Open\n",
        "ensures conn.state=Open\n",
        "=true\n",
        "λdisconnect(conn:Conn)=>Bool\n",
        "requires conn.state=Open\n",
        "ensures conn.state=Closed\n",
        "=true\n",
        "λmain()=>Bool=true",
    );
    let r = typecheck(source);
    assert!(r.is_ok(), "Valid state contracts should compile: {r:?}");
}

// ============================================================================
// State violations
// ============================================================================

#[test]
fn double_resolve_rejected() {
    let source = format!(
        concat!(
            "{}",
            "λdoubleResolve(ticket:Ticket)=>Bool={{",
            "l _=(resolve(ticket):Bool);",
            "resolve(ticket)",
            "}}"
        ),
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    assert!(r.is_err(), "Second resolve after first should fail");
    let e = r.unwrap_err();
    assert!(
        e.message.contains("requires clause") || e.message.contains("requires"),
        "Expected requires violation: {}",
        e.message
    );
}

#[test]
fn resolve_after_resolve_is_state_violation() {
    let source = format!(
        concat!(
            "{}",
            "λbadWorkflow(ticket:Ticket)=>Bool={{",
            "l _=(resolve(ticket):Bool);",
            "addNote(\"late note\",ticket)",
            "}}"
        ),
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    // addNote requires ticket.state=Open but after resolve it's Closed
    assert!(
        r.is_err(),
        "addNote after resolve should fail: ticket is Closed"
    );
}

// ============================================================================
// Error codes
// ============================================================================

#[test]
fn unknown_type_gives_proto_error() {
    let source = concat!(
        "protocol Phantom\n",
        "  Open → Closed via close\n",
        "  initial = Open\n",
        "  terminal = Closed\n",
        "λmain()=>Bool=true",
    );
    let r = typecheck(source);
    let e = r.unwrap_err();
    assert!(
        e.message.contains("SIGIL-PROTO-UNKNOWN-TYPE"),
        "Expected SIGIL-PROTO-UNKNOWN-TYPE, got: {}",
        e.message
    );
}

#[test]
fn missing_contract_gives_proto_error() {
    let source = concat!(
        "t Token={id:String}\n",
        "protocol Token\n",
        "  Open → Closed via consume\n",
        "  initial = Open\n",
        "  terminal = Closed\n",
        "λconsume(token:Token)=>Bool=true\n",
        "λmain()=>Bool=true",
    );
    let r = typecheck(source);
    let e = r.unwrap_err();
    assert!(
        e.message.contains("SIGIL-PROTO-MISSING-CONTRACT"),
        "Expected SIGIL-PROTO-MISSING-CONTRACT, got: {}",
        e.message
    );
}

// ============================================================================
// State propagation through let chains
// ============================================================================

#[test]
fn state_propagates_through_sequential_calls() {
    // addNote returns a Ticket in Open state via ensures result.state=Open.
    // Starting from a fresh type-ascribed ticket (initial = Open), calling
    // addNote preserves Open state, then resolve requires Open — should pass.
    let source = format!(
        concat!(
            "{}",
            "λworkflow(id:String)=>Bool={{",
            "l ticket=({{id:id}}:Ticket);",
            "l noted=addNote(\"note\",ticket);",
            "resolve(noted)",
            "}}"
        ),
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    assert!(
        r.is_ok(),
        "State propagates through sequential calls: {r:?}"
    );
}

#[test]
fn let_initial_state_is_available_inside_binary_expression() {
    let source = format!(
        concat!(
            "{}",
            "λworkflow(id:String)=>Bool={{",
            "l ticket=({{id:id}}:Ticket);",
            "ticket.id=id and resolve(ticket)",
            "}}"
        ),
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    assert!(
        r.is_ok(),
        "Initial protocol state should prove nested binary calls: {r:?}"
    );
}

#[test]
fn binary_expression_carries_left_state_transition_to_right_operand() {
    let source = format!(
        concat!(
            "{}",
            "λbad(ticket:Ticket)=>Bool\n",
            "requires ticket.state=Open\n",
            "ensures ticket.state=Closed\n",
            "=resolve(ticket) and resolve(ticket)"
        ),
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    let e = r.unwrap_err();
    assert!(
        e.message.contains("Call does not satisfy requires clause"),
        "Expected second binary operand to see Closed state, got: {}",
        e.message
    );
}

#[test]
fn via_function_must_require_and_ensure_declared_transition() {
    let source = concat!(
        "t Token={id:String}\n",
        "protocol Token\n",
        "  Open → Closed via consume\n",
        "  initial = Open\n",
        "  terminal = Closed\n",
        "λconsume(token:Token)=>Bool\n",
        "requires token.state=Open\n",
        "ensures token.state=Open\n",
        "=true\n",
        "λmain()=>Bool=true",
    );
    let r = typecheck(source);
    let e = r.unwrap_err();
    assert!(
        e.message.contains("SIGIL-PROTO-MISSING-CONTRACT")
            && e.message.contains("must ensure Token.state=Closed"),
        "Expected protocol transition contract mismatch, got: {}",
        e.message
    );
}

#[test]
fn runtime_protocol_state_field_is_rejected() {
    let source = format!(
        concat!(
            "{}",
            "λmain()=>Bool={{",
            "l ticket=({{id:\"T-1\"}}:Ticket);",
            "ticket.state",
            "}}"
        ),
        TICKET_PROTOCOL
    );
    let r = typecheck(&source);
    let e = r.unwrap_err();
    assert!(
        e.message.contains("Protocol state access is contract-only"),
        "Expected contract-only state access error, got: {}",
        e.message
    );
}

#[test]
fn runtime_state_label_is_rejected() {
    let source = format!("{}λmain()=>Bool=Open", TICKET_PROTOCOL);
    let r = typecheck(&source);
    let e = r.unwrap_err();
    assert!(
        e.message
            .contains("Protocol state label 'Open' is contract-only"),
        "Expected contract-only state label error, got: {}",
        e.message
    );
}

#[test]
fn imported_protocol_member_contracts_are_enforced() {
    let provider_source = format!("{}λmain()=>Bool=true", TICKET_PROTOCOL);
    let provider_tokens = tokenize(&provider_source).unwrap();
    let provider_program = parse(provider_tokens, "provider.sigil").unwrap();
    let provider_result = type_check(&provider_program, &provider_source, None).unwrap();
    let ticket_type_info = provider_program
        .declarations
        .iter()
        .find_map(|decl| match decl {
            sigil_ast::Declaration::Type(type_decl) if type_decl.name == "Ticket" => {
                Some(TypeInfo {
                    type_params: type_decl.type_params.clone(),
                    definition: type_decl.definition.clone(),
                    constraint: type_decl.constraint.clone(),
                    labels: Default::default(),
                })
            }
            _ => None,
        })
        .unwrap();

    let string_type = InferenceType::Primitive(TPrimitive {
        name: sigil_ast::PrimitiveName::String,
    });
    let bool_type = InferenceType::Primitive(TPrimitive {
        name: sigil_ast::PrimitiveName::Bool,
    });
    let ticket_type = InferenceType::Record(TRecord {
        fields: HashMap::from([("id".to_string(), string_type)]),
        name: Some("stdlib::ticket.Ticket".to_string()),
    });
    let resolve_type = InferenceType::Function(Box::new(TFunction {
        params: vec![ticket_type.clone()],
        return_type: bool_type,
        effects: None,
    }));
    let ticket_namespace = InferenceType::Record(TRecord {
        fields: HashMap::from([("resolve".to_string(), resolve_type)]),
        name: Some("stdlib::ticket".to_string()),
    });

    let options = TypeCheckOptions {
        imported_namespaces: Some(HashMap::from([(
            "stdlib::ticket".to_string(),
            ticket_namespace,
        )])),
        imported_type_registries: Some(HashMap::from([(
            "stdlib::ticket".to_string(),
            HashMap::from([("Ticket".to_string(), ticket_type_info)]),
        )])),
        imported_function_contracts: Some(HashMap::from([(
            "stdlib::ticket".to_string(),
            provider_result.function_contracts,
        )])),
        imported_protocol_registries: Some(HashMap::from([(
            "stdlib::ticket".to_string(),
            provider_result.protocol_registry,
        )])),
        ..TypeCheckOptions::default()
    };
    let source = concat!(
        "λbad(ticket:§ticket.Ticket)=>Bool={",
        "l _=(§ticket.resolve(ticket):Bool);",
        "§ticket.resolve(ticket)",
        "}"
    );
    let r = typecheck_with_options(source, options);
    let e = r.unwrap_err();
    assert!(
        e.message.contains("Call does not satisfy requires clause"),
        "Expected imported protocol requires violation, got: {}",
        e.message
    );
}
