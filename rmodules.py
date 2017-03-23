import os, string
def module(command, *arguments):
  commands = os.popen('/home/frlae/rust/rmodules/rmodules python %s %s'\
                      % (command, string.join(arguments))).read()
  exec commands
