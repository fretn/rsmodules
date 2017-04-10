import os
from rsmodules import module

module("load","blast")

print(os.environ["LOADEDMODULES"])

module("list","")

#this variable is declared in a module file with setenv("SOMEVAR","value")
#print(os.environ['SOMEVAR'])
