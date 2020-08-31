## Ceph Rust
### Official Ceph Rust interface

[![Build Status](https://github.com/ceph/ceph-rust/workflows/CI/badge.svg)](https://github.com/ceph/ceph-rust/actions?query=workflow%3ACI)
[![Version info](https://img.shields.io/crates/v/ceph.svg)](https://crates.io/crates/ceph)

Official Ceph Rust-lang interface. Contributions welcomed!

This library is the core librados Rust interface for Ceph. It also supports Admin Socket commands.

### Build requirements

Librados must be installed.

On CentOS/RHEL - Ceph Hammer librados is located in /usr/lib64. So, to get rust to see it you need to create a new symlink:
`sudo ln -s /usr/lib64/librados.so.2.0.0 /usr/lib64/librados.so`

On Ubuntu - Ceph Hammer librados is located in /usr/lib. So, to get rust to see it you need to create a new symlink:
`sudo ln -s /usr/lib/librados.so.2.0.0 /usr/lib/librados.so`

There may be another way to change the link name in rust without having to create a symlink.

On MacOS, you can install librados via homebrew:

```shell
$ brew tap zeichenanonym/ceph-client
$ brew install ceph-client
```

### Ceph
Create a Ceph development environment or use an existing Ceph environment.

If creating a Ceph environment then use the following. It will generate a 4 node Virtual Box Ceph system with one
node being a bootstrap node that controls the other. The remaining 3 nodes are Ceph nodes (Mons, OSDs, RGWs, APIs).

Created and manage github.com/ceph/ceph-chef (Chef cookbooks for Ceph) and the Bloomberg github.com link below for chef-bcs. Chef-bcs uses ceph-chef. These are the same tools  at Bloomberg.

Requirements for Mac OSX:
1. VirtualBox
2. git
3. Locate an area where you would like to install the Ceph build environment
4. git clone https://github.com/bloomberg/chef-bcs.git

```shell
$ cd chef-bcs
$ cd /bootstrap/vms/vagrant
$ ./CEPH_UP
```

**NOTE: If using the latest version of chef-bcs, you can enable an automatic development environment to be built with all of the development tools. See the project for details. It does it by default for Vagrant build.**

This will take about 30 minutes to build out. It installs CentOS 7.3, downloads all of the parts required to get Ceph up and running with good options.

Once complete you can then login to the first node:

`$ vagrant ssh ceph-vm1`

Run `ceph -s` to make sure you see Ceph running. Now you can install the development environment and Rust.

### Rust
(In ceph-vm1 node)
```
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```
OR
```
curl https://sh.rustup.rs -sSf | sh
```

### Yum
(In ceph-vm1 node) - Note: This is automatically done for you if you installed the environment vi Chef-bcs as noted above.

```
mkdir -p projects/lambdastack
cd projects/lambdastack

Requirements for development:
sudo yum install -y git cmake
sudo yum install -y openssl openssl-devel
```

Clone ceph-rust project:
```
git clone https://github.com/lambdastackio/ceph-rust.git
```

NOTE: Make sure you have setup your favorite editor. Vim is automatically installed.

### AWS S3 Object Store
Crate (library): aws-sdk-rust at https://github.com/lambdastackio/aws-sdk-rust

### AWS S3 CLI Utility
Crate (binary): s3lsio at https://github.com/lambdastackio/s3lsio

### Ceph Admin Commands

An example of finding a mon socket in a generic like environment.
```
ceph-conf --name mon.$(hostname -s) --show-config-value admin_socket
```

The raw admin_socket commands can be found in:
/src/ceph_admin_socket_mon_commands.json
/src/ceph_admin_socket_osd_commands.json
/src/ceph_admin_socket_client_commands.json

A number of them are the same.

------------
Portions originated from Chris Holcombe at https://github.com/cholcombe973
