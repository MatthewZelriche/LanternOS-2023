
.text
.type current_elx_synchronous, @function
current_elx_synchronous_stub:
   save_cpu_context current_elx_synchronous
   ret


.text
.type current_elx_irq, @function
current_elx_irq_stub:
   save_cpu_context current_elx_irq
   ret


.text
.type current_elx_fiq, @function
current_elx_fiq_stub:
   save_cpu_context current_elx_fiq
   ret


.text
.type current_elx_serror, @function
current_elx_serror_stub:
   save_cpu_context current_elx_serror
   ret