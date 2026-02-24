t Todo={id:ℤ,text:𝕊,done:𝔹}

λaddTodo(todos:[Todo],id:ℤ,text:𝕊)→[Todo]=[Todo{id:id,text:text,done:⊥}]⧺todos

λdeleteTodo(todos:[Todo],targetId:ℤ)→[Todo]=todos⊳λ(todo:Todo)→𝔹=todo.id≠targetId

λclearCompleted(todos:[Todo])→[Todo]=todos⊳λ(todo:Todo)→𝔹=¬todo.done

λtoggleTodo(todos:[Todo],targetId:ℤ)→[Todo]=todos↦λ(todo:Todo)→Todo≡todo.id=targetId{
  ⊤→Todo{id:todo.id,text:todo.text,done:¬todo.done}|
  ⊥→todo
}

λeditTodo(todos:[Todo],targetId:ℤ,newText:𝕊)→[Todo]=todos↦λ(todo:Todo)→Todo≡todo.id=targetId{
  ⊤→Todo{id:todo.id,text:newText,done:todo.done}|
  ⊥→todo
}

λcompletedCount(todos:[Todo])→ℤ=todos⊕(λ(acc:ℤ,todo:Todo)→ℤ≡todo.done{
  ⊤→acc+1|
  ⊥→acc
})⊕0

λlenTodos(todos:[Todo])→ℤ≡todos{
  []→0|
  [_,.rest]→1+lenTodos(rest)
}

test "todo add prepends item" {
  ≡addTodo([],1,"Task"){
    [todo]→todo.id=1∧todo.text="Task"∧todo.done=⊥|
    _→⊥
  }
}

test "todo toggle flips done flag" {
  toggleTodo([Todo{id:1,text:"Task",done:⊥}],1)[0].done=⊤
}

test "todo edit updates text" {
  editTodo([Todo{id:1,text:"Old",done:⊥}],1,"New")[0].text="New"
}

test "todo delete removes target" {
  ≡deleteTodo([Todo{id:1,text:"A",done:⊥},Todo{id:2,text:"B",done:⊥}],1){
    [todo]→todo.id=2∧todo.text="B"|
    _→⊥
  }
}

test "todo clearCompleted keeps active only" {
  ≡clearCompleted([Todo{id:1,text:"A",done:⊤},Todo{id:2,text:"B",done:⊥}]){
    [todo]→todo.id=2∧todo.done=⊥|
    _→⊥
  }
}

test "todo completedCount counts completed" {
  completedCount([Todo{id:1,text:"A",done:⊤},Todo{id:2,text:"B",done:⊥},Todo{id:3,text:"C",done:⊤}])=2
}

test "todo delete reduces length" {
  lenTodos(deleteTodo([Todo{id:1,text:"A",done:⊥},Todo{id:2,text:"B",done:⊥}],1))=1
}
