## 目的

宝飾用のプライスラベル作成

This module supports API for priting on multiple P-Touch seriese printers connected by USB ports.


Create a new printer by specifying the model and the serial number 

A printer driver for Brother P-Touch QL Series connected by USB Ports. 


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



Continous29
[2020-10-27T06:05:45Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 1D, A, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T06:05:45Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(Continuous(Continuous29)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

Continous62
[2020-10-27T05:58:53Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 3E, A, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T05:58:53Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(Continuous(Continuous62)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

Continuous62 Two-Color
[2020-10-27T05:59:55Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 3E, A, 0, 0, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 81, 0, 0, 0, 0, 0, 0]
[2020-10-27T05:59:55Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(Continuous(Continuous62)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

DieCut62x100
[2020-10-27T06:02:50Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 3E, B, 0, 0, 3, 0, 0, 1D, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T06:02:50Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(DieCut(DieCut62x29)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

DieCut29x90
[2020-10-27T06:07:59Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 1D, B, 0, 0, 1, 0, 0, 5A, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T06:07:59Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(DieCut(DieCut29x90)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

DieCut29x42
[2020-10-27T06:12:45Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 1D, B, 0, 0, 8, 0, 0, 2A, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T06:12:45Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(DieCut(DieCut29x42)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

DieCut23x23
[2020-10-27T06:14:31Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 17, B, 0, 0, C, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T06:14:31Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(DieCut(DieCut23x23)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable }

No Media
[2020-10-27T07:13:35Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T07:13:35Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: None, mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable, id: 0 }

Cover Open - No Media Installed
[2020-10-27T07:15:59Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T07:15:59Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(4096), media: None, mode: 0, status_type: Error, phase: Receiving, notification: NotAvailable, id: 0 }

Cover Open - Media Installed
[2020-10-27T07:27:53Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 10, 3E, A, 0, 0, 15, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T07:27:53Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: CoverOpen, media: Some(Continuous(Continuous62)), mode: 0, status_type: Error, phase: Receiving, notification: NotAvailable, id: 21 }

Cover Closed - Media Installed - Not Inserted to Feeding Slit
[2020-10-27T07:30:06Z DEBUG ptouch::printer] Raw status code: [80, 20, 42, 34, 38, 30, 0, 0, 0, 0, 3E, A, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
[2020-10-27T07:30:06Z DEBUG ptouch::printer] Parsed Status struct: Status { model: QL800, error: UnknownError(0), media: Some(Continuous(Continuous62)), mode: 0, status_type: ReplyToRequest, phase: Receiving, notification: NotAvailable, id: 21 }