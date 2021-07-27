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

#include <stdlib.h>
#include <xtensa/config/core.h>

#include "xtensa_rtos.h"

#include "FreeRTOS.h"
#include "task.h"

// Defined in portasm.h
extern void _frxt_tick_timer_init(void);

// Defined in xtensa_context.S
extern void _xt_coproc_init(void);

#ifndef configCORETIMER
#define configCORETIMER 0
#endif

// Duplicate of inaccessible xSchedulerRunning; needed at startup to avoid
// counting nesting
volatile uint32_t port_scheduler_running[configNUM_CORES] = {0};
// Interrupt nesting level. Increased/decreased in portasm.c,
// _frxt_int_enter/_frxt_int_exit
volatile uint32_t port_interrupt_nesting[configNUM_CORES] = {0};

// User exception dispatcher when exiting
void _xt_user_exit(void);

// Implemented in rust
// void vPortYieldCore(int xOtherCoreID) {}

/*
 * See header file for description.
 */
StackType_t *pxPortInitialiseStack(StackType_t *pxTopOfStack,
                                   TaskFunction_t pxCode, void *pvParameters,
                                   BaseType_t xRunPrivileged) {
    StackType_t *sp, *tp;
    XtExcFrame *frame;

    (void)xRunPrivileged;

#if XCHAL_CP_NUM > 0
    uint32_t *p;
#endif

    /* Create interrupt stack frame aligned to 16 byte boundary */
    sp = (StackType_t *)(((UBaseType_t)pxTopOfStack - XT_CP_SIZE -
                          XT_STK_FRMSZ) &
                         ~0xf);

    /* Clear the entire frame (do not use memset() because we don't depend on C
     * library) */
    for (tp = sp; tp <= pxTopOfStack; ++tp) {
        *tp = 0;
    }

    frame = (XtExcFrame *)sp;

    /* Explicitly initialize certain saved registers */
    frame->pc = (UBaseType_t)pxCode; /* task entrypoint */
    frame->a0 = 0;                   /* to terminate GDB backtrace		*/
    frame->a1 = (UBaseType_t)sp +
                XT_STK_FRMSZ;                 /* physical top of stack frame		*/
    frame->exit = (UBaseType_t)_xt_user_exit; /* user exception exit dispatcher
                                               */

/* Set initial PS to int level 0, EXCM disabled ('rfe' will enable), user mode.
 */
/* Also set entry point argument parameter. */
#ifdef __XTENSA_CALL0_ABI__
#if CONFIG_FREERTOS_TASK_FUNCTION_WRAPPER
    frame->a2 = (UBaseType_t)pxCode;
    frame->a3 = (UBaseType_t)pvParameters;
#else
    frame->a2 = (UBaseType_t)pvParameters;
#endif
    frame->ps = PS_UM | PS_EXCM;
#else
    /* + for windowed ABI also set WOE and CALLINC (pretend task was 'call4'd).
     */
    frame->a6 = (UBaseType_t)pvParameters;
    frame->ps = PS_UM | PS_EXCM | PS_WOE | PS_CALLINC(1);
#endif /* ifdef __XTENSA_CALL0_ABI__ */

#ifdef XT_USE_SWPRI
    /* Set the initial virtual priority mask value to all 1's. */
    frame->vpri = 0xFFFFFFFF;
#endif

#if XCHAL_CP_NUM > 0
    /* Init the coprocessor save area (see xtensa_context.h) */

    /* No access to TCB here, so derive indirectly. Stack growth is top to
     * bottom.
     * //p = (uint32_t *) xMPUSettings->coproc_area;
     */
    p = (uint32_t *)(((uint32_t)pxTopOfStack - XT_CP_SIZE) & ~0xf);
    configASSERT((uint32_t)p >= frame->a1);
    p[0] = 0;
    p[1] = 0;
    p[2] =
        (((uint32_t)p) + 12 + XCHAL_TOTAL_SA_ALIGN - 1) & -XCHAL_TOTAL_SA_ALIGN;
#endif

    return sp;
}

/*
 * See header file for description.
 */
BaseType_t xPortStartScheduler(void) {
    /* Interrupts are disabled at this point and stack contains PS with enabled
     * interrupts when task context is restored */

#if XCHAL_CP_NUM > 0
    /* Initialize co-processor management for tasks. Leave CPENABLE alone. */
    _xt_coproc_init();
#endif

    UBaseType_t core = portGET_CORE_ID();

    //   if (core == 0) {
    /* Init the tick divisor value */
    _xt_tick_divisor_init();
    /* Setup the hardware to generate the tick. */
    _frxt_tick_timer_init();
    //   }

    port_scheduler_running[core] = 1;

    /* Cannot be directly called from C; never returns */
    asm volatile("call0 _frxt_dispatch");

    /* Should not get here. */
    return pdTRUE;
}

void vPortEndScheduler(void) { /* Do not implement. */
}

BaseType_t xPortSysTickHandler(void) {
    /* Interrupts upto configMAX_SYSCALL_INTERRUPT_PRIORITY must be
     * disabled before calling xTaskIncrementTick as it access the
     * kernel lists. */
    BaseType_t ret = xTaskIncrementTick();
    portYIELD_FROM_ISR(ret);

    return ret;
}

#if portUSING_MPU_WRAPPERS
void vPortStoreTaskMPUSettings(xMPU_SETTINGS *xMPUSettings,
                               const struct xMEMORY_REGION *const xRegions,
                               StackType_t *pxBottomOfStack,
                               uint32_t usStackDepth) {
    (void)xRegions;
#if XCHAL_CP_NUM > 0
    xMPUSettings->coproc_area =
        (StackType_t *)((((uint32_t)(pxBottomOfStack + usStackDepth - 1)) -
                         XT_CP_SIZE) &
                        ~0xf);

/* NOTE: we cannot initialize the coprocessor save area here because FreeRTOS is
 * going to clear the stack area after we return. This is done in
 * pxPortInitialiseStack().
 */
#endif
}

void vPortReleaseTaskMPUSettings(xMPU_SETTINGS *xMPUSettings) {
    /* If task has live floating point registers somewhere, release them */
    _xt_coproc_release(xMPUSettings->coproc_area);
}

#endif