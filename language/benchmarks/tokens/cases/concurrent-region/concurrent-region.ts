type ConcurrentOutcome<T,E>=
  | {kind:"success",value:T}
  | {kind:"failure",message:E}
  | {kind:"aborted"};

type Result<T,E>={ok:true,value:T}|{ok:false,error:E};

function sleepMs(ms:number):Promise<void>{
  return new Promise((resolve)=>setTimeout(resolve,ms));
}

async function main():Promise<ConcurrentOutcome<number,string>[]>{
  return Promise.all([1,2,3].map(runProcess));
}

async function process(value:number):Promise<Result<number,string>>{
  await sleepMs(0);
  return {ok:true,value};
}

async function runProcess(value:number):Promise<ConcurrentOutcome<number,string>>{
  const result=await process(value);
  return result.ok ? {kind:"success",value:result.value} : {kind:"failure",message:result.error};
}
