# Fluid Simulation (GPU-Based SPH)

## Overview

This project is a real-time fluid simulation built using **Smoothed Particle Hydrodynamics (SPH)** and accelerated on the GPU.

## Technologies Used

* **wgpu** (GPU abstraction layer)
* **WGSL** (shader language)
* **Rust** (host-side logic)

---

## Controls

Hold and drag to roate the camera around the origin. W and S zooms in and out. Space starts the sim. R restarts the sim. M toggles 
the modifications window where you can customize the parameters in real time. 

## Installation

Clone the repo. Have cargo installed and you can run with cargo run 
```bash
git clone https://github.com/LucasVand/fluid.git
cd fluid
cargo run --release
```

## Acknowledgements

* Sebastian Lague, Inspired by his video
