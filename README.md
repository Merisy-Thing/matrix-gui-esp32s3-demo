# Matrix GUI ESP32-S3 Demo

A Matrix GUI demonstration project running on ESP32-S3 microcontroller.

## Demo

![Demo](esp32s3/assets/demo.gif)

## Web Demo

Try the interactive demo online in your browser:

![Launch Web Demo](https://merisy-thing.github.io/matrix-gui-esp32s3-demo)

The web version uses WebAssembly to run the Matrix GUI framework in the browser, providing the same experience as the embedded version without requiring hardware.

### Building the Web Demo

```bash
cd web
trunk serve --open
```

This will start a local web server and open the demo in your browser.

## Hardware

- ESP32-S3
- ST7789 LCD display
- CST816D touch controller

## Dependencies

- [matrix-gui](https://github.com/Merisy-Thing/matrix-gui) - GUI framework
- multi-mono-font - Font rendering with multi-language support
- embedded-graphics - 2D graphics library
