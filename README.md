# Multi-platform Tray Indicator

Please see the [examples](https://github.com/olback/tray-item-rs/tree/master/examples) as documentation is currently lacking.

Tray Indicator uses icons from gresources on Linux and `.rc`-files on Windows.  
These recourses have to be packed into be final binary.

* [x] Linux
* [x] Windows
* [x] MacOS*

\* MacOS does not allow running applications in threads other than main, meaning that
it is not possible to listen on events in a new thread. See the `macos.rs` example for a how-to.

### Todo:
* [ ] Docs
