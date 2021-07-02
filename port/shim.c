#include "FreeRTOS.h"
#include "task.h"

UBaseType_t ulTaskEnterCriticalFromISR() {
    return taskENTER_CRITICAL_FROM_ISR();
}

void vTaskExitCriticalFromISR(UBaseType_t uxSavedInterruptStatus) {
    taskEXIT_CRITICAL_FROM_ISR(uxSavedInterruptStatus);
}

size_t strlen(const char* str) {
    return __builtin_strlen(str);
}
