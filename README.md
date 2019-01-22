## RSModules

The RSModules package is a tool to help you to modify your environment
by using modulefiles.  
A modulefile contains all the settings needed to configure your shell to be able to use a certain application. A modulefile can modify environment variables, execute commands or even load other modules for dependencies.

You modify your environment ```$PATH, $LD_LIBRARY_PATH, $PYTHONPATH, $PERL5LIB, ...``` by loading and/or unloading modules.  
You don't have to worry about manually modifying your ```$PATH``` or other environment variables when installing (multiple versions of) a package.   
The moment you need a version or a package, just load the module.

The philosophy behind RSModules is to provide module implementation where speed is important and which is very easy
to setup and administer.

## Table of Contents
1. [Example](#example)
2. [Features](#features)
3. [Installation](#Installation)
  - [Manual Installation](#manual-installation)
  - [Automatic Installation](#automatic-installation)
  - [Compiling from source](#compiling-from-source)
4. [Modulefiles](#modulefiles)
 * [Example modulefile](#example-modulefile)
5. [Using from perl](#using-from-perl)
6. [Using from python](#using-from-python)
7. [PAQ](#paq)

### Example
An example says more than a thousand words:

```bash
[user@awesome ~]$ echo $PATH
/usr/local/bin:/bin:/usr/bin:/usr/local/sbin:/usr/sbin:/sbin
[user@awesome ~]$ which perl
/usr/bin/perl
[user@awesome ~]$ module av perl

  Module name | Description
              |                     
  perl/5.10.1 | 
D perl/5.14.1 | Perl 5.14.1 is a highly capable, feature-rich programming language with over 29 years of development.
  perl/5.8.9  | 


  * D means that the module is set as the default module.
  * Loaded modules are printed in bold.

[user@awesome ~]$ module load perl
[user@awesome ~]$ which perl
/usr/local/perl/5.14.1/bin/perl
[user@awesome ~]$ echo $PATH
/usr/local/perl/5.14.1/bin:/usr/local/bin:/bin:/usr/bin:/usr/local/sbin:/usr/sbin:/sbin
[user@awesome ~]$ 
```
## Features

 * ```module available [--default] [search string]``` Shows all the (default) modules or the modules that match the search string.
 * ```module info [(partial)modulename] [(partial)modulename] [...]``` Shows info about the requested module(s).
 * ```module load [(partial)modulename] [(partial)modulename] [...]``` Loads the requested modules.
 * ```module switch [(partial)modulename from] [(partial)modulename to] ``` Switch between the requested modules.
 * ```module unload [(partial)modulename] [(partial)modulename] [...]``` Unloads the requested modules.
 * ```module purge``` Unloads all loaded modules.
 * ```module refurbish``` Unloads all loaded modules and reloads all autoloaded modules
 * ```module refresh``` Reloads all loaded modules.
 * ```module undo``` Undo previous load/unload/switch/purge actions
 * ```module list``` Shows a list of all the loaded modules.
 * ```module autoload append|prepend|remove|list|purge [modulename(s)]``` Manages the auto loading of modules by adding them to your startup scripts.
 * ```module delete [modulename(s)]``` Deletes one or more modulefiles. But only if you have the permissions to do so.
 * ```module readme [modulename]``` Looks for a manpage or README file in the module installation folder and displays the contents of this file.
 * The output is not redirected to stderr, but to stdout. So you are able to use grep / rg on the output and it doesn't trigger errors in pipelines.
 * RSModules is fast because it's written in a compiled language and it is using cache files for listing the modules.
 * By using module info, users can easily discover what a module provides and how they use the software that is bundled with the module.
 * The RSModules binary has no dependencies. RSModules works on linux and macOS.

```bash
[user@awesome ~]$ ldd rsmodules
	not a dynamic executable
[user@awesome ~]$ file rsmodules
rsmodules: ELF 64-bit LSB executable, x86-64, version 1 (GNU/Linux), statically linked, BuildID[sha1]=d46b88517c5a31aa8241b8cac1dd48f96c04d26c, stripped
[user@awesome ~]$ 

```
 * Support for bash, zsh, tcsh, csh, perl and python.
 * Tabcompletion support for bash, tcsh and csh.
 
## Installation

### Manual installation
RSModules has a few configuration options which are located in the setup scripts.

 * ```$RSMODULES_INSTALL_DIR```: the environment variable points to the location where rsmodules is installed
 * ```$MODULEPATH```: the environment variable that contains a ```:``` separated list of folders where module files can be found.
 
The scripts ```setup_rsmodules.sh``` and ```setup_rsmodules.sh``` need to be sourced when the user logs in. You either symlink these in /etc/profile.d/ or source them in the users init scripts (.bashrc, .cshrc, .zshrc, ...)

After you have added modulefiles, don't forget to update the module cache by running the command: ```update_modules_cache```

In case you want to use RSModules inside your python or perl scripts, add the ```$RSMODULES_INSTALL_DIR``` to the ```$PYTHONPAH``` and/or ```$PERL5LIB``` environment
variable(s). But this shouldn't be needed as the ```setup_rsmodules.(c)sh``` scripts do this for you.

### Automatic installation
RSModules has a simple installation wizard.

 * First of all make sure that the environment variables ```$RSMODULES_INSTALL_DIR``` and ```$MODULEPATH``` are not set.
 * Then run the ```rsmodules``` command and the installation wizard will guide you around.

Depending on your permissions rsmodules will be either installed in your home directory or system wide.

The wizard sets ```$RSMODULES_INSTALL_DIR``` and ```$MODULEPATH``` in the ```setup_rsmodules.(c)sh``` files. For the root user these files will be symlinked in ```/etc/profile.d/```. For non root users these files will be sourced from your .bashrc, .cshrc.
On the next login the ```module``` command will be available

A dummy module will be created to help you around.

After you have added modulefiles, don't forget to update the module cache by running the command: ```update_modules_cache```

### Compiling from source
First you'll have to install Rust on your system:
```bash
[user@awesome ~]$ curl https://sh.rustup.rs -sSf | sh
```
Now run the following command in the root of the project: 

```bash
[user@awesome ~]$ cargo build --release
```

The ```rsmodules``` binary can be found in the ```target/release/``` folder.

Or you can create a distributable .tar.gz by running:
```bash
[user@awesome ~]$ ./create_dist.sh
```
The resulting .tar.gz file can be found in the ```releases/``` folder.

## Modulefiles

Modulefiles are very simple scripts. They are based on the [rhai](https://github.com/jonathandturner/rhai#rhai-language-guide) [https://github.com/jonathandturner/rhai#rhai-language-guide] scripting syntax.

Next to the default rhai syntax, the following functions are available:

 * ```setenv("variable","value");```
 * ```getenv("variable");```
 * ```unsetenv("variable");```
 * ```prepend_path("variable","value");```
 * ```append_path("variable","value");```
 * ```remove_path("variable","value");```
 * ```system("command");```
 * ```load("modulename");```
 * ```unload("modulename");```
 * ```conflict("modulename");```
 * ```description("module description");```
 * ```set_alias("name","value");```
 * ```is_loaded("modulename");```
 * ```source("shelltype", "/path/to/filename.shell-extension");```

### Example modulefile

```lua
description("InterProScan is the software package that allows sequences to be scanned against InterPro's signatures");
description("");
description("InterPro is a resource that provides functional analysis of protein sequences by classifying them into");
description("families and predicting the presence of domains and important sites. To classify proteins in this way,");
description("InterPro uses predictive models, known as signatures, provided by several different databases,");
description("(referred to as member databases), that make up the InterPro consortium.");

conflict("iprscan");

load("java/1.8.0_60");
load("perl");
load("python/2.7.2");
load("EMBOSS");
load("gcc");

prepend_path("PATH","/software/shared/apps/iprscan/5.23-62/bin/");
prepend_path("PATH","/software/shared/apps/iprscan/5.23-62/");

system("mkdir -p /scratch/tmp/iprscan_logs");
system("mkdir -p /scratch/tmp/iprscan_tmp");

source("bash", "/software/shared/apps/iprscan/5.23-64/env.sh");
source("zsh", "/software/shared/apps/iprscan/5.23-64/env.zsh");

var x = getenv("SOMEVAR");

if x == "OKAY" {
    load("abyss");
} else {
    load("R");
    unsetenv("SOMEVAR");
}

if is_loaded("blast/2.2.17") {
    load("blast/2.5.0+");
}

```
###### note: As the time of writing rhai scripts don't support tabs, only spaces.

After you have created new modulefiles, don't forget to update the module cache by running the command:
```bash
[user@awesome ~]$ module makecache
```

You can also have a progress bar while updating the cache.
```bash
[user@awesome ~] update_modules_cache
```

### Using from perl

When you source the file ```setup_rsmodules.(c)sh``` there is a path added to your ```$PERL5LIB``` environment
variable. This gives you the possibility to use rsmodules from inside your perl scripts.

```perl
#!/usr/bin/perl

use rsmodules;

module("load blast");

print($ENV{"LOADEDMODULES"} . "\n");

module("list");

# this var is set with setenv("SOMEVAR","value") in the modulescript
print($ENV{"SOMEVAR"});
```

### Using from python

When you source the file ```setup_rsmodules.(c)sh``` there is a path added to your ```$PYTHONPATH``` environment
variable. This gives you the possibility to use rsmodules from inside your python scripts.

```python
import os
from rsmodules import module

module("load","blast")

print(os.environ["LOADEDMODULES"])

module("list","")

#this variable is declared in a module file with setenv("SOMEVAR","value")
print(os.environ['SOMEVAR'])
```

## PAQ

#### What does PAQ mean ?

Possibly asked questions :)

#### Why RSModules while tclmodules/cmodules and Lmod are around?

A couple of years ago I ran into some issues with tcl/c modules. Because I was annoyed by the fact that
everything was printed to stderr, I started writing my own implementation of modules in C. When 90% of the
features where ready (90% as in the other 10% takes also another 90% time ;) ). I found Lmod, so I stopped
working on my little sideproject and started to try Lmod.

It was more or less decided to switch to Lmod. Until I was looking for a new sideproject to learn the
programming language Rust. In no time I had 90% ;) of the code ready + some features our users have been
asking for. (a way to see which executables a module provides, a short description in the module av output).

And that's how RSmodules was born.

#### Does this mean that tcl/c modules or Lmod are bad projects ?

Absolutely not ! Feel free to use whatever that suits your needs.

#### Why then didn't you contribute to these projects instead ?

I needed a side project to learn Rust and the project got a bit out of hand :)
 
#### What does the RS in RSModules mean ?

RS stands for [rust](https://www.rust-lang.org)
 
#### How fast is RSModules ?

 
```bash
[user@awesome ~]$ time module makecache

/software/shared/rsmodulefiles/ was succesfully indexed.

  * Total number of modules: 927
  * Number of default (D) modules: 151

real	0m4.781s
user	0m0.062s
sys   	0m0.168s
[user@awesome ~]$
```
###### The module files in the above example are located on a network share

#### Can I convert my tcl modulefiles to rhai modulefiles ?

Robert McLay, the Lmod developer, was so kind to write a script to translate tcl modulefiles to
lua modulefiles. 
I have modified his script to output rhai modulefiles instead. 
So to answer the question, yes its possible. 
But its not possible to provide tcl module files in your ```$MODULEPATH``` and
let RSModules translate them on the fly.

#### Will you add tabcompletion for zsh ?

Maybe, if I ever figure out how it works, feel free to contribute.
 
#### What happens when I load a module that is already loaded ?

The module is first unloaded and then reloaded again. 

#### What happens when I load a different version of an already loaded module ?

The module is replaced with the newly loaded module:

```bash
[user@awesome ~]$ module load python
[user@awesome ~]$ module list

  Currently loaded modules:

  * python/3.5.1

[user@awesome ~]$ module load python/2.7.2

  The previously loaded module python/3.5.1 has been replaced with python/2.7.2

[user@awesome ~]$ module list

  Currently loaded modules:

  * python/2.7.2

[user@awesome ~]$
```

This is basicly the same as the ```module switch [from modulename] [to modulename]``` command.
#### I want to autoload some modules everytime I login. What do I need to do ?

The command ```module autoload``` is what you are looking for.

#### I want to remove a module through a script, but I don't want the interactive mode.

Call ```rsmodules``` directly and supply 'noshell' as the shell, example:
```bash
[user@awesome ~]$ /usr/local/bin/rsmodules noshell delete module/1.0 module/2.1
Removal of module/1.0 module/2.1 was succesful. Don't forget to update the module cache.
[user@awesome ~]$
```
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
 
