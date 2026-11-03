# VulkanMod Horizon 26.3 port status

This archive contains only the VulkanMod source files supplied for the port. No AppleSkin or other mod files are included.

## Metadata alignment
- Minecraft target: 26.3
- Horizon repository: https://github.com/horizon-rust/horizon
- Horizon revision: 0bcd2f15678532560589773f674fb6290b075005

## Source audit
- Active native hooks: 4 (mod list, tick, settings, settings click)
- Disabled/unmapped injections: 228
- TODO markers in mixin configuration: 226
- Settings UI and settings click handlers are placeholders.
- Core Vulkan rendering integration is not yet implemented.

The source was generated from VulkanMod 0.6.8+26.1.2. Updating package labels and compatibility metadata does not constitute a port of its rendering implementation to Minecraft 26.3.

## Build status
Not compiled or tested. Do not treat this as a working Vulkan renderer yet.
