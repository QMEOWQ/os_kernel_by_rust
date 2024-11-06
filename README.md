## A rust os kernel

### all codes tested on ubuntu 20.04 are successful.

### The implementation order is:

```
1.  rust mini kernel & vga_buffer & print_console ->
2.  test framework ->
3.  simple_interrupts ->
4.  double_fault ->
5.  simple_outer_interrupts ->
6.  l4_table ->
7.  memory_allocator->
8.  heap_allocator_by_crate ->
9.  allocator_by_hand->
10. async/await
```

### Some shots of the project which follow the order above:

1.

![alt text](PixPin_2024-09-02_17-06-56.png)
![alt text](PixPin_2024-09-03_14-27-11.png)
![alt text](PixPin_2024-09-15_17-36-08.png)
![alt text](PixPin_2024-09-25_11-32-20.png)

2.
    None.

3.
![alt text](PixPin_2024-09-15_17-36-08-1.png)

4.
![alt text](PixPin_2024-09-17_22-58-06.png)

5.
![alt text](PixPin_2024-09-19_12-55-53.png)

6.
![alt text](PixPin_2024-09-22_19-54-13.png)

7.
![alt text](PixPin_2024-09-19_09-35-16.png)

8.
![alt text](PixPin_2024-09-27_20-02-11.png)

9.
![alt text](PixPin_2024-09-25_11-14-12.png) 
![alt text](PixPin_2024-09-25_10-36-33.png)

10.
![alt text](PixPin_2024-10-02_15-51-55.png) 
![alt text](PixPin_2024-10-02_14-02-07.png) 
![alt text](PixPin_2024-10-02_15-01-25.png)


### build and run:

- you should use `rust-nightly` version.
  eg.

```
rustup install nightly
rustup install nightly
rustc --version
```

- cmd only used once(when you first try to build the project):

1.

```
rustc --version --verbose, look for current host machine information.
rustup target add thumbv7em-none-eabihf, a ARM embedded system without an os.
cargo build --target thumbv7em-none-eabihf, compile current project as an executable file for an ARM embedded system.
```

2.

```
The following two commands compile to select the local operating system as the target.
Linux: cargo rustc -- -C link-arg=-nostartfiles
Windows: cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"
```

3.

```
cargo build --target x86_64-os_by_rust.json
rustup component add rust-src
```

4.

```
cargo install bootimage
rustup component add llvm-tools-preview
cargo bootimage, compile customed kernel.
```

then when you update your code, you just need to run:

```
cargo bootimage
cargo build
cargo run
```

how to activate your kernel:

1. activate your kernel in your virtual machine

```
qemu-system-x86_64 -drive format=raw,file=target/x86_64-os_by_rust/debug/bootimage-os_by_rust.bin
```

2. activate your kernel in your real machine
   <span style="color: red;">(! sdX is the device name of the USB flash drive, and you must be extremely careful when selecting the device name, because all the existing data on the target device will be erased.)</span>

```
dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
```

### notion:

We will put the original version in main branch, and the updated version in dev branches.
