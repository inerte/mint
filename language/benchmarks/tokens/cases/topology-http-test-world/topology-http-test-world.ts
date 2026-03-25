import { mailerApi } from "./topology";
import { systemClock } from "./world/clock";
import { realFs } from "./world/fs";
import { proxy } from "./world/http";
import { capture } from "./world/log";
import { deny } from "./world/process";
import { buildWorld } from "./world/runtime";
import { virtual } from "./world/timer";

export const world=buildWorld(systemClock(),realFs(),[proxy("http://127.0.0.1:45110",mailerApi)],capture(),deny(),[],virtual());
