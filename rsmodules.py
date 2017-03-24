import os, string
def module(command, *arguments):
  commands = os.popen(os.environ['RSMODULES_INSTALL_DIR'] + '/rsmodules python %s %s'\
                      % (command, string.join(arguments))).read()
  exec commands
