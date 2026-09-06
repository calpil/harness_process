# Plan - Feature #78: el instalador respalda sus scripts pero no el backlog, que es lo unico irrecuperable

Estado: in_progress
Microservicios:
- harness_process (setup_harness.sh / setup_harness.ps1)

## Alcance

Tres cambios en los dos instaladores, y el smoke que los prueba en rojo primero.

## Peldano de huella

`Peldano elegido: 1 (extender lo que existe) porque se quitan dos entradas de una
lista, se agrega un paso de respaldo con el mecanismo que ya existe (backup_file)
y se le pone voz a una siembra que ya existe.` Sin flag nuevo: la decision del
usuario fue que --force no cambie nada para los datos, asi que no hay que
inventar `--keep-data` ni nada parecido.

## Delegacion (implementer)

- D-1 (AC-5, PRIMERO): el smoke gana un bloque que planta features y reglas,
  corre reinstalar / --reset / --reset --force / archivo faltante, y afirma. Se
  corre ANTES del fix para ver el rojo: hoy --reset borra y la siembra calla.
- D-2 (AC-1): sacar `feature_list.json` y `progress` de `reset_targets` (sh) y
  del `$targets` del reset (ps1).
- D-3 (AC-2): `backup_data` — respaldo de backlog + progress/ al inicio de toda
  corrida, con `backup_file` pero SIN mirar FORCE. En el ps1, lo mismo con
  `Backup-HarnessPath`.
- D-4 (AC-3, AC-4): la siembra solo-si-falta avisa en [WARN] con los respaldos
  que encuentra (bkp/ y docs/bkp-backlog/) y el comando de restauracion.
- D-5 (AC-6, AC-7): paridad, UPDATING.md y --help.

## Criterios de cierre (reviewer)

- Cada aserto nuevo del smoke tiene que fallar contra el instalador de HEAD.
- Ningun aserto puede pasar por casualidad: el backlog plantado tiene features
  Y reglas distintas de la plantilla, y se compara byte a byte.

## Riesgos

- R-1: el instalador corre sobre proyectos con `bkp/` grande; un respaldo mas por
  corrida de tres archivos chicos es despreciable.
- R-2: el `--reset` de instalaciones viejas que CONTABAN con borrar el backlog.
  No hay evidencia de que alguien lo usara para eso; se documenta en UPDATING.

## Observaciones (decisiones pendientes)

- OBS-1 (DECIDIDA 2026-09-06): --force no saltea el respaldo de datos.
- OBS-2: la rama de integracion se pregunta antes de `close --status done --to`.

---
Cerrado: 2026-09-06T22:53:40Z - status=done - 
