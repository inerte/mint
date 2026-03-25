from topology import mailer_api
from world.clock import system_clock
from world.fs import real
from world.http import proxy
from world.log import capture
from world.process import deny
from world.runtime import build_world
from world.timer import virtual


world = build_world(system_clock(), real(), [proxy("http://127.0.0.1:45110", mailer_api)], capture(), deny(), [], virtual())
