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
- Exige el flujo completo del arnes aunque pida 'fix it now': add -> start -> spec -> aprobacion -> implementar; la urgencia no saltea la aprobacion. (#77)
- Los commits van sin trailers de IA (Co-Authored-By, Claude-Session); prefirio reescribir un commit ya publicado antes que dejarlo con uno. (#78)
- Prefiere limites duros sin escape por flag (sin --force, sin motivo que saltee el tope): un escape se vuelve el default. (#17, #80)
- Quiere decidir los forks de diseno antes de que se implemente: cada OBS se le presenta con recomendacion y el elige. (#72, #75, #80)
- Un aviso mide crecimiento desde la ultima accion del usuario, no el acumulado historico; un umbral que hay que subir para callarlo no mide nada. (#80, #82)
