## 目的

This module supports API for priting on multiple P-Touch seriese printers connected by USB ports.


Create a new printer by specifying the model and the serial number 


```
/home/yako/Projects/node-ptouch-usb/node_modules/usb/usb.js:38
	this.__open()
	     ^

Error: LIBUSB_ERROR_ACCESS
```

`LIBUSB_ERROR_ACCESS`とエラーが発生する

```sh
$ sudo gpasswd -a $USER plugdev
```

## udevルールを追加

```sh:/etc/udev/rules.d/50-ptouch.rules
SUBSYSTEMS=="usb", ATTRS{idVendor}=="04f9", ATTRS{idProduct}=="209b|209c|209d", MODE="664", GROUP="plugdev"
```

```
sudo udevadm trigger
sudo udevadm control --reload-rules
```


https://static.dev.sifive.com/dev-tools/FreedomStudio/2019.08/freedom-studio-manual-4.7.2-2019-08-1.pdf

https://gill.net.in/posts/reverse-engineering-a-usb-device-with-rust/