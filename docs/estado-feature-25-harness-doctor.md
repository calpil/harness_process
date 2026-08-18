# Estado archivado - Feature #25: harness_doctor
Cerrada: 2026-08-17T19:28:53Z - status=done - doctor diagnostica la instalacion (siete areas) con el comando exacto de remedio por problema; exit 2 solo si algo impide trabajar. La escalera de la #24 partio la feature: peldano 3 para el diagnostico, peldano 1 (el lanzador) para la mitad que un doctor dentro del binario no puede cubrir.

---

# Feature #25: harness_doctor

Estado: in_progress
Plan: docs/plan-feature-25-harness-doctor.md
Spec: docs/spec-feature-25-harness-doctor.md

Microservicios:
- harness

Evidencia:
- 
- 2026-08-17T19:04:42Z Plan de la #25 escrito: D1-D8 citando cada AC. Primera aplicacion real de la escalera de la #24 y sale hibrida: peldano 3 para el diagnostico (con la razon escrita de por que el 1 y el 2 no alcanzaban: el 1 convertiria una instalacion incompleta en bloqueo del proceso, el 2 obligaria a reimplementar en shell la resolucion de rutas que costo la #10) y peldano 1 para el arreglo del lanzador. Las siete areas diagnostican fallas que este repo YA sufrio.
- 2026-08-17T19:28:32Z Feature #25 implementada: harness_cli doctor diagnostica las siete areas de la instalacion con remedio por linea, y el lanzador cubre la mitad que un doctor dentro del binario no puede (binario ausente o viejo). La escalera de la #24 partio la feature en dos peldanos distintos. Hallazgo: el chequeo del hub estuvo a punto de cerrar con un OK falso (TCP conecta, las operaciones morian con connection reset); la linea ahora dice exactamente que midio.
