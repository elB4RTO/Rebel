<div align="center">
  <h1>Rebel</h1>
  <b>An indipendent 64-bit kernel</b>
</div>

<br/><br/>

## Dependencies

To compile Rust sources, the target `x86_64-unknown-none`  must be available

```
rustup target add x86_64-unknown-none
```

<br/><br/>

## Try it out

One command to rule them all

```
make
```

#### Compile the sources

Compile and link all the needed sources. Objects will be placed inside the *build* directory

```
make build
```

#### Create the disk image

The disk image will be created in the project's root directory and will have a size of approximately 100 MB

```
make create
```

#### Run in a virtual machine

At the moment the only officially supported emulator is `bochs`. Despite that, other emulators can be used as well

```
make run
```

#### Clean up

The *build* folder and the disk image will be permanently deleted

```
make clean
```

<br/><br/>

<div align="center">
  <i>Long Live Rebels</i>
</div>