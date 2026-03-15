# Side Invaders

**Side Invaders** is a small embedded arcade game written in **Rust** for the **ESP32-C3** and a **128×64 OLED display**.

The game is loosely inspired by the classic **Space Invaders**, but with a twist: the battlefield is **rotated sideways**. Instead of enemies descending from above, they approach from the side, forcing the player to maneuver vertically while firing across the screen.

Your goal is simple: **survive as long as possible and achieve the highest score.**

---

## Gameplay

* You start with **3 lives**.
* You can have **up to 10 projectiles active at once**.a
* Destroy enemies to **increase your score**.
* Lose all your lives and the game ends.

---

## Hardware Requirements

The game is designed for a simple microcontroller setup.

| Component       | Description                               |
| --------------- | ----------------------------------------- |
| Microcontroller | ESP32-C3                                  |
| Display         | 128×64 OLED (SSD1306 / SSD1315 compatible) |
|                 | SCL: GPIO1, SDA: GPIO0                    |
| Inputs          | 5 push buttons (GPIO{3, 2, 5, 6, 7})      |

Button usage:

| Buttons |
| ------- |
| Up      |
| Down    |
| Left    |
| Right   |
| Fire    |

Joystick support is planned for future versions.

---

## Software

Side Invaders is written in **Rust** using a **`no_std` embedded environment**.

Main libraries used:

* `esp-hal`
* `embedded-graphics`
* `ssd1306`


## Building

Build the project using Cargo for the ESP32-C3 target:

```bash
cargo build --release
```

Flash the program to the device using your preferred flashing tool.

---

## License

This project is intended for experimentation and learning with **embedded Rust**.

  
