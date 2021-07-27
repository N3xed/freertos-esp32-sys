/*
 * FreeRTOS Kernel <DEVELOPMENT BRANCH>
 * Copyright (C) 2015-2019 Cadence Design Systems, Inc.
 * Copyright (C) 2021 Amazon.com, Inc. or its affiliates.  All Rights Reserved.
 *
 * SPDX-License-Identifier: MIT
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * https://www.FreeRTOS.org
 * https://github.com/FreeRTOS
 *
 */

#ifndef PORTMACRO_H
#define PORTMACRO_H

/* Architecture specifics. These can be used by assembly files as well. */
#define portSTACK_GROWTH            (-1)
#define portBYTE_ALIGNMENT          4
#define portCRITICAL_NESTING_IN_TCB 1
#ifndef configNUM_CORES
#define configNUM_CORES 2
#endif
// Needed for core pinning when coprocessors are used.
#define configUSE_CORE_AFFINITY 1
#define portINTLEVEL_HIGHINT    5
#define portMPU_SETTINGS_SIZE   4

#ifndef __ASSEMBLER__

#include <stdint.h>

// #include <xtensa/tie/xt_core.h>
#include <xtensa/config/core.h>
#include <xtensa/config/system.h> /* required for XSHAL_CLIB */
#include <xtensa/hal.h>

#ifdef STRUCT_FIELD
#undef STRUCT_FIELD
#endif
#ifdef STRUCT_AFIELD
#undef STRUCT_AFIELD
#endif
#include "xtensa_context.h"
#include <xtensa/xtruntime.h>

// #ifdef STRUCT_FIELD
// #undef STRUCT_FIELD
// #endif
// #include "xtensa_context.h"

// Function / variable declarations

size_t strlen(const char *str);

void _frxt_setup_switch(void);
void _xt_coproc_release(volatile uint32_t *coproc_area);
void _xt_coproc_init(void);

void vPortYieldCore(int xOtherCoreID);
void vPortYield(void);

void vTaskEnterCritical(void);
void vTaskExitCritical(void);

void vPortTakeISRLock(void);
void vPortGiveISRLock(void);
void vPortTakeTaskLock(void);
void vPortGiveTaskLock(void);

extern volatile uint32_t port_scheduler_running[configNUM_CORES];
extern volatile uint32_t port_interrupt_nesting[configNUM_CORES];

// Type definitions

#define portSTACK_TYPE uint32_t
typedef portSTACK_TYPE StackType_t;
typedef int32_t BaseType_t;
typedef uint32_t UBaseType_t;

#if (configUSE_16_BIT_TICKS == 1)
typedef uint16_t TickType_t;
#define portMAX_DELAY 0xffff
#else
typedef uint32_t TickType_t;
#define portMAX_DELAY           0xffffffffUL

// 32-bit tick type on a 32-bit architecture, so reads of the tick count do
// not need to be guarded with a critical section.
#define portTICK_TYPE_IS_ATOMIC 1
#endif

#define portTICK_PERIOD_MS ((TickType_t)1000 / configTICK_RATE_HZ)

/* Scheduler utilities. */
#define portYIELD() vPortYield()
#define portNOP()   XT_NOP()

static inline void portYIELD_FROM_ISR(BaseType_t xSwitchRequired) {
    if (xSwitchRequired) {
        _frxt_setup_switch();
    }
}

/* SMP utilities. */
static inline BaseType_t portGET_CORE_ID() {
    BaseType_t id;
    asm volatile("rsr.prid %0\n\t"
                 "extui %0,%0,13,1"
                 : "=r"(id));
    return id;
}
#define portYIELD_CORE(x) vPortYieldCore(x)

/* Architecture specific optimisations. */
#ifndef configUSE_PORT_OPTIMISED_TASK_SELECTION
#define configUSE_PORT_OPTIMISED_TASK_SELECTION 0

// When coprocessors are defined, we to maintain a pointer to coprocessors area.
// We currently use a hack: redefine field xMPU_SETTINGS in TCB block as a
// structure that can hold: MPU wrappers, coprocessor area pointer, trace code
// structure, and more if needed. The field is normally used for memory
// protection. FreeRTOS should create another general purpose field.
typedef struct {
#if XCHAL_CP_NUM > 0
    // Pointer to coprocessor save area; MUST BE FIRST
    volatile StackType_t *coproc_area;
#endif

#if portUSING_MPU_WRAPPERS
    // Define here mpu_settings, which is port dependent
#error Not implemented
#endif
} __attribute__((packed)) xMPU_SETTINGS;

// Main hack to use MPU_wrappers even when no MPU is defined (warning:
// mpu_setting should not be accessed; otherwise move this above xMPU_SETTINGS)
#if (XCHAL_CP_NUM > 0) && !portUSING_MPU_WRAPPERS // If MPU wrappers not used, we still
                                                  // need to allocate coproc area
#undef portUSING_MPU_WRAPPERS
#define portUSING_MPU_WRAPPERS 1 // Enable it to allocate coproc area
#define MPU_WRAPPERS_H           // Override mpu_wrapper.h to disable unwanted code
#define PRIVILEGED_FUNCTION
#define PRIVILEGED_DATA
#endif
#endif

#if configUSE_PORT_OPTIMISED_TASK_SELECTION == 1

/* Store/clear the ready priorities in a bit map. */
#define portRECORD_READY_PRIORITY(uxPriority, uxReadyPriorities)                         \
    (uxReadyPriorities) |= (1UL << (uxPriority))
#define portRESET_READY_PRIORITY(uxPriority, uxReadyPriorities)                          \
    (uxReadyPriorities) &= ~(1UL << (uxPriority))

#define portGET_HIGHEST_PRIORITY(uxTopPriority, uxReadyPriorities)                       \
    uxTopPriority = (31UL - (uint32_t)__builtin_clz(uxReadyPriorities))

#endif /* configUSE_PORT_OPTIMISED_TASK_SELECTION */

// Critical section management.

/*
 * This differs from the standard portDISABLE_INTERRUPTS()
 * in that it also returns what the interrupt state was
 * before it disabling interrupts.
 */
#define portDISABLE_INTERRUPTS() XTOS_SET_INTLEVEL(portINTLEVEL_HIGHINT)

#define portENABLE_INTERRUPTS() XTOS_SET_INTLEVEL(0)

/*
 * Will restore the interrupt mask to the ulState obtained from
 * `portDISABLE_INTERRUPTS()`.
 */
static inline void portRESTORE_INTERRUPTS(UBaseType_t ulState) {
    asm volatile("WSR.PS %0" : : "r"(ulState) : "memory");
}

/*
 * Returns non-zero if currently running in an
 * ISR or otherwise in kernel mode.
 */
static inline BaseType_t portCHECK_IF_IN_ISR() {
    BaseType_t status = portDISABLE_INTERRUPTS();
    BaseType_t in_interrupt = port_interrupt_nesting[portGET_CORE_ID()] != 0;
    portRESTORE_INTERRUPTS(status);
    return in_interrupt;
}

#define portASSERT_IF_IN_ISR() configASSERT(portCHECK_IF_IN_ISR() == 0)

#define portGET_ISR_LOCK()      vPortTakeISRLock()
#define portRELEASE_ISR_LOCK()  vPortGiveISRLock()
#define portGET_TASK_LOCK()     vPortTakeTaskLock()
#define portRELEASE_TASK_LOCK() vPortGiveTaskLock()

#define portENTER_CRITICAL() vTaskEnterCritical()
#define portEXIT_CRITICAL()  vTaskExitCritical()

/*
 * vTaskEnterCritical() has been modified to be safe to use
 * from within ISRs. Returns the current interrupt mask.
 */
static inline UBaseType_t portSET_INTERRUPT_MASK_FROM_ISR() {
    UBaseType_t mask = portDISABLE_INTERRUPTS();
    vTaskEnterCritical();
    return mask;
}

/*
 * vTaskExitCritical() has been modified to be safe to use
 * from within ISRs.
 */
static inline void portCLEAR_INTERRUPT_MASK_FROM_ISR(UBaseType_t x) {
    vTaskExitCritical();
    portRESTORE_INTERRUPTS(x);
}

/* Runtime stats support */
#if (configGENERATE_RUN_TIME_STATS == 1)
#define portCONFIGURE_TIMER_FOR_RUN_TIME_STATS() /* nothing needed here */
#define portGET_RUN_TIME_COUNTER_VALUE()         xthal_get_ccount()
#endif

/* Maps sprintf and snprintf to the lite version in lib_rtos_support */
#if (configUSE_DEBUG_SPRINTF == 1)
#define sprintf(...)  rtos_sprintf(__VA_ARGS__)
#define snprintf(...) rtos_snprintf(__VA_ARGS__)
#endif

#define portTIMER_CALLBACK_ATTRIBUTE

/* Task function macros as described on the FreeRTOS.org WEB site.  These are
not necessary for to use this port.  They are defined so the common demo files
(which build with all the ports) will build. */
#define portTASK_FUNCTION_PROTO(vFunction, pvParameters)                                 \
    void vFunction(void *pvParameters)
#define portTASK_FUNCTION(vFunction, pvParameters) void vFunction(void *pvParameters)

#endif /* __ASSEMBLER__ */

#endif /* PORTMACRO_H */
