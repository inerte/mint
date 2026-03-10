#!/bin/bash
set -euo pipefail

echo "========================================"
echo "TCP Client/Server Integration Tests"
echo "========================================"
echo ""

PROJECT_DIR="tcp-roundtrip-project"
PORT="45120"
SERVER_LOG="server.log"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
    wait "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${PROJECT_DIR}"
}

trap cleanup EXIT

rm -rf "${PROJECT_DIR}"
mkdir -p "${PROJECT_DIR}/src"

cat > "${PROJECT_DIR}/sigil.json" << 'EOF'
{
  "layout": {
    "src": "src",
    "tests": "tests",
    "out": ".local"
  }
}
EOF

cat > "${PROJECT_DIR}/src/tcpRoundtripServer.sigil" << EOF
i stdlib⋅tcpServer

λhandleRequest(request:stdlib⋅tcpServer.Request)→!IO stdlib⋅tcpServer.Response match request.message{
  "ping"→stdlib⋅tcpServer.response("pong")|
  "upper:hello"→stdlib⋅tcpServer.response("HELLO")|
  _→stdlib⋅tcpServer.response(request.message)
}

λmain()→!IO Unit=stdlib⋅tcpServer.serve(handleRequest,${PORT})
EOF

cat > "${PROJECT_DIR}/src/pingClient.sigil" << EOF
i stdlib⋅tcpClient

λmain()→!IO String match stdlib⋅tcpClient.send("127.0.0.1","ping",${PORT}){
  Ok(response)→response.message|
  Err(error)→"ERR:"++error.message
}
EOF

cat > "${PROJECT_DIR}/src/echoClient.sigil" << EOF
i stdlib⋅tcpClient

λmain()→!IO String match stdlib⋅tcpClient.send("127.0.0.1","echoed",${PORT}){
  Ok(response)→response.message|
  Err(error)→"ERR:"++error.message
}
EOF

cat > "${PROJECT_DIR}/src/upperClient.sigil" << EOF
i stdlib⋅tcpClient

λmain()→!IO String match stdlib⋅tcpClient.send("127.0.0.1","upper:hello",${PORT}){
  Ok(response)→response.message|
  Err(error)→"ERR:"++error.message
}
EOF

cat > "${PROJECT_DIR}/src/invalidClient.sigil" << EOF
i stdlib⋅tcpClient

λmain()→!IO String match stdlib⋅tcpClient.send("", "ping", ${PORT}){
  Ok(response)→response.message|
  Err(error)→error.message
}
EOF

cd "${PROJECT_DIR}"
../target/debug/sigil run src/tcpRoundtripServer.sigil > "${SERVER_LOG}" 2>&1 &
SERVER_PID=$!

node - <<EOF
const net = require('node:net');
const port = ${PORT};
let tries = 0;
function attempt() {
  tries += 1;
  const socket = net.createConnection({ host: '127.0.0.1', port }, () => {
    socket.end();
    process.exit(0);
  });
  socket.once('error', () => {
    socket.destroy();
    if (tries >= 50) process.exit(1);
    setTimeout(attempt, 200);
  });
}
attempt();
EOF

if [[ $? -ne 0 ]]; then
  echo "TCP server did not start"
  cat "${SERVER_LOG}" 2>/dev/null || true
  exit 1
fi

run_and_assert() {
  local file=$1
  local expected=$2
  local output
  output=$(../target/debug/sigil run "${file}" --human)
  echo "${output}"
  if ! grep -q "${expected}" <<<"${output}"; then
    echo "Expected '${expected}' from ${file}"
    exit 1
  fi
}

run_and_assert src/pingClient.sigil "pong"
run_and_assert src/echoClient.sigil "echoed"
run_and_assert src/upperClient.sigil "HELLO"
run_and_assert src/invalidClient.sigil "valid host and port"

cd ..

echo ""
echo "========================================"
echo "TCP integration tests complete!"
echo "========================================"
