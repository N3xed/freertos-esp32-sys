bindgen --use-core --size_t-is-usize --ctypes-prefix chlorine wrapper.h -o src/bindings.rs -- -I"port" -I"port/esp32" -I"dep/xtensa/esp32/include" -I"dep/xtensa/include" -I"dep/FreeRTOS-Kernel/include" --sysroot=../../tools/gcc/xtensa-esp32-elf -target mipsel-none-none-elf -DXTRUNTIME_H -DXTOS_RESTORE_JUST_INTLEVEL=