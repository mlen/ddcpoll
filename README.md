# `ddcpoll`

Attaching and detaching USB devices from a virtual machine used for gaming is quite annoying.
First of all, to detach them one does need to run a command on the host somehow.

`ssh` works, but it's not very convenient, so let's make something better!

This tool is an attempt to make it a little better. It polls displays' input source status via DDC
and runs appropriate command when input gets switched.

Combined with `virt-usb` tool, which wraps `libvirt` API, USB devices can follow input source selection!

## Configuration

Currently ddcpoll expects to find `config.toml` file in its working directory.
This will be fixed at some point.

The configuration format is quite simple TOML file:

```toml
# DELL
[[displays]]
serial = "GH85D64F019S"
feature = 96

  # primary output for desktop
  [[displays.actions]]
  value = 16
  command = "./virt-usb --detach --domain gamez --devices 1532:001c 0f39:0825"

  # secondary output for gamez
  [[displays.actions]]
  value = 15
  command = "./virt-usb --attach --domain gamez --devices 1532:001c 0f39:0825"
```

## License

This project is licensed under [MIT license](LICENSE).
