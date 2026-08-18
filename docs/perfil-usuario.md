# Perfil de usuario

Como quiere trabajar el usuario de este repositorio. Lo escribe el arnes
**solo con su si explicito** (`harness_cli perfil add --yes`) y el instalador
lo inyecta en las superficies que lee cada agente al arrancar.

Que va aca: preferencias durables sobre COMO trabajar (que elegir ante un
fork, que exigir antes de cerrar, que estilo de trabajo espera).
Que NO va: hechos de una feature puntual (eso es `docs/lecciones/`),
datos personales, y jamas un secreto — este archivo se versiona.

Limite duro: 1500 caracteres contando solo las entradas. Al pasarse, el
comando falla y hay que consolidar: nunca se recorta nada en silencio.

Entradas (una por linea, empezando con `- `):

- Ante un fork de consistencia o concurrencia, elige la opcion segura aunque cueste mas. (#14, #16)
- Prefiere features amplias y completas antes que incrementales: amplia el spec en vez de partirlo. (#15, #16)
- Ante un gate, prefiere bloquear a avisar cuando el error es caro o irreversible. (#17, #19)
- Exige sincronia total con sistemas externos, incluido el backfill de lo ya cerrado. (#15, #16)
