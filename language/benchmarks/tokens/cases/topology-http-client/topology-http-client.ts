import { emptyHeaders, get } from "./http-client";
import { mailerApi } from "./topology";

export async function main():Promise<string>{
  const result=await get(mailerApi,emptyHeaders(),"/health");
  return result.ok ? `${result.value.status}:${result.value.body}` : `ERR:${result.error.message}`;
}
