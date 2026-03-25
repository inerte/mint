import { println } from "./io";
import { contains } from "./test-log";
import { capture } from "./world-log";

export function main():void{}

export function testWorldsCaptureLogs():boolean{
  const log=capture();
  println(log,"temporary");
  return contains(log,"temporary");
}

export function testWorldsStartFresh():boolean{
  const log=capture();
  return !contains(log,"temporary");
}
