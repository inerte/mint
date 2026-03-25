from http_client import empty_headers, get
from topology import mailer_api


def main() -> str:
    result = get(mailer_api, empty_headers(), "/health")
    return f"{result['value']['status']}:{result['value']['body']}" if result["ok"] else f"ERR:{result['error']['message']}"
