# vfio_motion
A Rust-based (to replace a ridiculously slow chain of AutoHotkey + PowerShell + Node.js + Python scripts) server / client to manage fast attachment and detachment of Linux input devices to and from QEMU-based VMs under `libvirt`. To be used in tandem with input device sharing software such as [barrier](https://github.com/debauchee/barrier).

Note: This is a __work in progress__ - I haven't even started working on the client (Windows) side yet!
