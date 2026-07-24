# Constitution del proyecto

Principios no negociables del proyecto. Toda spec (`docs/spec-feature-*.md`) y
todo plan (`docs/plan-feature-*.md`) deben cumplirlos; el reviewer verifica su
cumplimiento antes de cada cierre.

> Documento del USUARIO: el instalador lo siembra una sola vez y nunca lo pisa.
> Ajusta los articulos a tu proyecto; los agentes lo leen y lo respetan, pero
> no lo editan.

## Articulo 1 - Calidad y tests primero

- Todo cambio llega con tests cercanos al codigo tocado y pasa los comandos
  oficiales de verificacion del proyecto.
- No se cierra una feature con tests rotos, saltados o sin ejecutar.

## Articulo 2 - Specs aprobadas antes de implementar

- Ninguna implementacion arranca sin spec (`docs/spec-feature-<id>-<slug>.md`)
  con `Estado: approved`.
- La DECISION de aprobar es exclusiva del USUARIO. El agente no decide: MUESTRA
  el spec (contenido en el chat y abierto en el editor del usuario), PREGUNTA si
  lo aprueba y solo con su SI explicito REGISTRA la aprobacion ejecutando
  `harness_cli approve-spec --yes` (que sella quien/cuando y re-firma el spec).
- PROHIBIDO aprobar sin ese si: ningun agente corre `approve-spec --yes` por
  iniciativa propia, ni edita a mano la linea `Estado:` para saltear el flujo.

## Articulo 3 - Trazabilidad AC-n

- Cada item de la Delegacion del plan cita el AC-n del spec que cubre.
- La evidencia de implementacion y el veredicto del reviewer se organizan por
  AC-n; un criterio sin evidencia es un criterio no cumplido.

## Articulo 4 - Seguridad y observabilidad minimas

- Sin secretos en el repo, en logs ni en commits; credenciales solo via
  entorno o configuracion ignorada por git.
- Errores accionables (que fallo y que hacer a continuacion) y exit codes
  estables en toda herramienta o servicio.

## Articulo 5 - Las decisiones del usuario mandan

- Ante observaciones o forks de diseno sin decision registrada, se pregunta al
  usuario ANTES de implementar y la respuesta queda registrada en el plan/spec.
- Ningun articulo de este documento se relaja sin decision explicita del
  usuario.

## Articulo 6 - Reglas puente a ADRs (ejemplo; ajusta a tu proyecto)

- Ninguna dependencia nueva de runtime sin un ADR que la justifique.
- Todo cambio de contrato compartido entre microservicios registra su impacto
  antes de mergear.
