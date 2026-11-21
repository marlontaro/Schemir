# Roadmap de Schemir

Estado honesto del proyecto: qué está hecho, qué falta y qué está sin probar.
_Última actualización: noviembre 2026._

## ✅ Hecho

- **Cuatro motores:** SQL Server (`tiberius`), PostgreSQL (`tokio-postgres`, con TLS),
  MySQL / MariaDB (`mysql_async`, con TLS si el servidor lo ofrece) y SQLite (`rusqlite`).
  MySQL y MariaDB hablan el mismo protocolo y publican los mismos catálogos, así que un solo
  lector cubre los dos; en ellos el esquema ES la base de datos.
- **Comparación de objetos:** tablas, columnas, vistas, procedimientos, funciones,
  claves primarias, claves foráneas, índices, restricciones CHECK y **disparadores (triggers)**.
- **Multiesquema:** lee y compara todos los esquemas de la base.
- **Opciones de comparación** (estilo SSDT): ignorar mayúsculas/minúsculas, espacios,
  comentarios y el valor por defecto de columnas, elegir qué tipos de objeto comparar y
  bloquear ante posible pérdida de datos.
- **Script de sincronización** (`CREATE` / `ALTER` / `DROP`) por dialecto, con
  **sincronización selectiva** (marcar/desmarcar qué objetos entran). Copiar y guardar `.sql`.
- **Bloquear ante posible pérdida de datos** (activado por defecto, como en SSDT): antes de
  quitar una columna o una tabla, o de cambiar el tipo de una columna, el script lleva
  delante un guardia que lo detiene si la tabla afectada tiene filas. Cada motor a su manera:
  `RAISERROR` en SQL Server, `RAISE EXCEPTION` en PostgreSQL, un CHECK violado en SQLite y un
  NOT NULL violado en MySQL / MariaDB (con `STRICT_ALL_TABLES` forzado, porque sin modo
  estricto el aviso no frenaría nada).
- **Aplicar al destino:** ejecutar el script (solo lo marcado) contra la base B desde la
  app, con confirmación y dentro de una transacción en SQL Server, PostgreSQL y SQLite
  (comprobado contra una base real en SQLite y en SQL Server). En MySQL / MariaDB va
  sentencia a sentencia y SIN transacción: el motor hace *commit* implícito en cada DDL, así
  que allí el guardia impide el daño pero no lo deshace.
- **Columnas que se autogeneran**, en los cuatro motores: el `IDENTITY` de SQL Server (con
  su inicio y su incremento), las columnas de identidad y las `serial` de PostgreSQL, el
  `AUTO_INCREMENT` de MySQL / MariaDB y el `AUTOINCREMENT` de SQLite. Se leen del catálogo,
  se comparan y se vuelven a emitir en el dialecto del destino; cuando el motor destino no
  puede expresar lo que trae el origen, el script lo dice en un comentario `-- (!)` en vez de
  perderlo en silencio, que es lo que pasaba antes.
- **Conectar primero y elegir la base después**, como SSMS y SSDT. `Probar conexión` conecta
  al SERVIDOR (ignorando lo que haya en el campo de base de datos, que es justo lo que no se
  sabe todavía) y trae la lista de las que el login puede ABRIR de verdad, no solo ver:
  `HAS_DBACCESS` en SQL Server, `has_database_privilege` en PostgreSQL y sin los esquemas de
  sistema en MySQL. El campo pasa a ser un desplegable; si el login no puede enumerarlas,
  vuelve a ser un campo de texto para no dejar a nadie atrapado. En SQLite no aplica: ahí la
  base es un archivo.
- **La interfaz habla de ORIGEN y DESTINO, no de A y B.** Las tarjetas de conexión ya decían
  origen y destino y los resultados decían A y B, obligando a traducir mentalmente.
- **Conectarse a SQL Server como se conecta SSMS**, por ODBC. El lector quedó partido en tres:
  las consultas, un parseo puro sobre un tipo de fila propio (`db::row::Row`) y dos transportes
  que traducen a ese tipo. Por un lado ODBC (`odbc-api`, dependencia solo de `cfg(windows)`),
  que habla memoria compartida, canalizaciones con nombre y TCP, y resuelve él mismo la
  autenticación de Windows; por otro tiberius sobre TCP, en Rust puro, que es el que queda
  fuera de Windows o cuando no hay controlador instalado. Se elige el controlador entre los
  instalados de verdad (`Environment::drivers()`), de 18 a «SQL Server», y el puerto solo se
  escribe en la cadena si NO es el 1433: escribirlo forzaría TCP, que es justo lo que se quiere
  evitar. `Probar conexión` dice por cuál de los dos entró. El refactor trajo de regalo las
  PRIMERAS pruebas del lector de SQL Server (el `Row` y la cadena de conexión), que antes no
  se podían escribir sin un servidor delante.
- **Refrescar los módulos que dependen de una tabla que cambia** (SQL Server). El motor
  resuelve las referencias de vistas y rutinas de forma diferida y les guarda los metadatos en
  caché: al cambiar la forma de una tabla, toda vista montada sobre ella sigue anunciando las
  columnas de antes. El modelo gana `Routine.references`, que el lector de SQL Server llena
  desde `sys.sql_expression_dependencies` resolviendo el nombre por `referenced_id` (y no por
  `referenced_entity_name`, que trae el nombre tal como lo escribió quien programó la vista).
  El generador emite `EXECUTE sp_refreshsqlmodule` para el cierre transitivo sobre las tablas
  que cambian de forma, por capas —primero las de abajo, porque una vista montada sobre otra
  tiene que refrescarse después—, saltándose los módulos que el propio script recrea, que
  nacen compilados. El cierre se itera hasta que una capa no agrega nada, así un ciclo de
  referencias entre procedimientos no lo cuelga. Medido contra una base real: el catálogo da
  47 módulos alcanzados, 10 los recrea el script y Schemir emite los 37 restantes, sin ninguno
  de más.
- **Emparejar por estructura lo que bautiza el motor.** SQL Server le pone nombre a las
  restricciones declaradas sin nombre y les pega un sufijo distinto en cada base
  (`UQ__Product__F25D9D35E79EE34A` frente a `UQ__Product__F25D9D356FD9DE35`), de modo que dos
  bases creadas con el mismo script mostraban «diferencias» que no existían. Ahora el modelo
  guarda `system_named` —lo dice el catálogo: `is_system_named` de `sys.key_constraints`,
  `sys.check_constraints` y `sys.foreign_keys`—, y el emparejamiento va primero por nombre y
  después, solo para esos, por la forma del objeto (tabla, columnas, expresión). Basta que el
  motor haya nombrado a uno de los dos lados. Lo que nombró una persona se sigue comparando
  por nombre: renombrar un índice propio SÍ es un cambio. Medido contra una base real, el
  comparador bajó de 21/9 a 18/6 objetos y el script perdió las 6 sentencias que sobraban.
  PostgreSQL y MySQL no lo necesitan: sus nombres automáticos son deterministas.
- **Autenticación de Windows en SQL Server**, que es como se conecta SSDT por defecto:
  la bandera `winauth` de tiberius y su `AuthMethod::Integrated`. Dentro de tiberius es una
  dependencia opcional y solo de `cfg(windows)`, así que encenderla no añade nada al build de
  macOS ni de Linux, y en los demás sistemas la función devuelve un error legible en vez de
  no compilar. La casilla de la interfaz apaga usuario y contraseña, y la bandera se recuerda
  con la conexión: no hay ninguna contraseña que guardar.
- **Comparar datos de filas** (SQLite): por tabla, empareja por clave primaria y reporta
  filas solo en A / solo en B / distintas, con su DML (`INSERT` / `UPDATE` / `DELETE`).
- **Diagrama ER** con renderizador propio (dagre): vista por esquema y **diagrama de
  diferencias** a color. Interactivo (zoom, desplazamiento, arrastrar, pantalla completa)
  y **exportable a PNG/SVG**.
- **Informe de diferencias** en HTML (imprimible a PDF) con resumen, detalle y script.
- **Modelador al estilo de ERwin:** el tipo de dato se **elige** del catálogo del motor
  (`crates/core/src/types.rs`) y el diseñador sabe si lleva longitud (`varchar(50)`), las
  dos cifras de precisión y escala (`decimal(18,2)`) o nada. Las tablas pertenecen a un
  **esquema** que se puede crear y asignar, se arrastran por el lienzo, y **la relación se
  tira arrastrando** de una columna a otra tabla: al soltar sale el diálogo que la nombra
  y le pone `ON DELETE` / `ON UPDATE`.
- **La relación migra la clave, como en ERwin:** se arrastra del punto de la clave del
  PADRE a la tabla hija. Si la hija no tiene esa columna, se crea sola: mismo nombre,
  hereda el tipo de la columna del padre —si no coincidieran, el motor no dejaría crear
  la clave foránea—, `NOT NULL` y fuera de la clave primaria, o sea una relación NO
  identificadora. El nombre por defecto se lee como una frase: `fkUserroleHasUser`. Si la
  hija ya tiene esa columna se usa la suya y no se le toca el tipo, y nada se crea hasta
  que se acepta el diálogo: cancelar no deja columnas sueltas.
- **La clave compuesta migra entera:** la relación se lleva TODAS las columnas de la clave
  del padre y no una, porque son la misma clave vista desde la hija y partirla no
  significa nada. El diálogo enseña una fila por columna y la foránea sale
  `FOREIGN KEY (a, b) REFERENCES t (a, b)`. Arrastrando de una columna que NO es parte de
  la clave migra solo esa. La relación identifica a la hija cuando la clave ENTERA entra
  en su primaria: con una parte dentro y otra fuera la fila todavía necesita algo más, y
  eso no es identificar.
- **Si cambia la clave del padre, la foránea la sigue:** la primaria del padre y la que
  migró a la hija son la misma clave vista dos veces, así que meter o sacar una columna de
  la primaria mueve la foránea con ella —la que entra baja a la hija con su tipo, la que
  sale deja de ser foránea, y reordenar la clave reordena el `FOREIGN KEY (a, b)`—. Lo que
  sale de la clave NO se borra de la hija: la columna sigue ahí con sus datos. Se respetan
  las relaciones puestas a propósito sobre otra columna, y las de una tabla consigo misma,
  donde no hay respuesta automática que valga. Si el padre se queda sin clave ninguna no se
  toca nada, porque eso casi siempre es un paso intermedio mientras se edita.
- **Borrar una columna la saca de las relaciones.** Antes la relación seguía nombrándola:
  la marca `(FK)` se perdía y el script salía con una foránea sobre un nombre inventado.
  La relación que se queda sin ningún par se va con ellos.
- **Áreas temáticas: un recorte del modelo**, las *subject areas* de ERwin. Con cuarenta
  tablas en el lienzo, la conversación de hoy es sobre seis; un área guarda una lista de
  tablas y **nada más**, así que no es una copia ni un modelo aparte: borrar un área no
  borra ninguna tabla, y una tabla puede estar en varias a la vez. **Cada área coloca sus
  tablas a su manera**, como los diagramas de ERwin —en «Ventas» pones seis donde se lean
  y en «Almacén» la misma `producto` cae en otro sitio—; con una sola posición global,
  abrir un área de seis tablas dejaba la pantalla llena de los huecos de las otras
  treinta y cuatro, y entonces el recorte no servía para lo único que sirve: leerse. La
  primera vez que una tabla aparece en un área se cae a su sitio del modelo entero, así
  que el área empieza como ya estaba menos lo que no se ve. El **tamaño** sí es uno solo:
  lo ancha que tiene que ser una tabla para que se lean sus columnas es igual en todas
  partes. **El script NO cambia:** el SQL se genera siempre del modelo entero, porque un
  script que dependiera de dónde estás mirando sería una trampa —la foránea a una tabla
  que quedó fuera fallaría al ejecutarlo— y es la misma regla que ya siguen lógico y
  físico. Lo mismo la validación: un problema en una tabla que ahora no se ve rompe el
  script igual, así que se sigue listando, y pulsarlo **sale del recorte** para llevarte a
  ella. Dentro de un área la ✕ **quita del área**, no borra del modelo: lo que se quita de
  una vista no se borra de lo que la vista mira. Una tabla creada con un área delante
  entra en ella sola, porque si no nacería fuera de lo que estás viendo. Las relaciones
  con un extremo fuera no se pueden dibujar, así que **la barra las cuenta**: no verlas se
  parece demasiado a que no existan. El editor enseña el modelo completo con una casilla
  por tabla, un filtro, y **«+ las relacionadas»**, que trae las que tienen una relación
  directa con las marcadas —un salto y no todos, porque encadenando se acaba trayendo el
  modelo entero, que es justo lo que el área venía a evitar—.
- **El modelo se valida contra el catálogo del motor.** La lista que ERwin saca al validar,
  con la diferencia que es la razón de ser de Schemir: aquí el motor y su versión forman
  parte de la pregunta. `varchar` sin longitud se escribe igual de bien en cualquier parte y
  SQL Server lo acepta dándole UNO, así que la tabla se crea sin una queja y el error
  aparece meses después al guardar el primer nombre; en PostgreSQL ese mismo `varchar` es
  legítimo y significa «sin límite». Eso no lo ve quien lee el script: lo ve quien conoce el
  catálogo, y el catálogo ya está cargado. Encuentra tipos que el motor no tiene, tipos que
  exigen una versión más nueva que la elegida —y dice **cuál** columna, que es lo que la
  nota de la barra no decía—, longitudes que faltan o que pasan del máximo, precisión y
  escala imposibles, columnas repetidas, tablas sin nombre, sin columnas o sin primaria, y
  foráneas con los dos lados de distinto tipo, apuntando a media clave o a una tabla que no
  tiene ninguna. Dos niveles y no cinco: **error** es lo que el motor va a rechazar,
  **aviso** es lo que funcionará pero probablemente no es lo que se quería. Nada de esto
  BLOQUEA generar el script —un diseñador que no deja mirar el SQL hasta que todo esté
  perfecto estorba justo cuando más falta hace ver por dónde va la cosa—: los errores se
  escriben en la cabecera del script, así que si alguien lo copia y el motor lo rechaza, el
  propio script dice por qué. En el árbol salen al final y **solo si hay alguno**, porque una
  rama que anuncia cero problemas en todos los modelos deja de leerse, y entonces tampoco se
  lee el día que marca tres. Un clic lleva al objeto y dos lo abren para arreglarlo.
- **Frases verbales: el diagrama se puede leer en voz alta.** Dos por relación, una por
  sentido, porque las dos se leen y no dicen lo mismo: «Pedido de venta contiene Línea del
  pedido» y «Línea del pedido pertenece a Pedido de venta». Cada una va pegada al extremo
  del que habla —en medio no se sabría cuál es cuál— y con un borde del color del lienzo
  por debajo, que es lo que las deja legibles cuando cruzan otra línea. Salen en lógico,
  donde el árbol nombra la relación por su frase y no por un nombre de restricción que no
  le dice nada a quien la está leyendo. El diálogo las pide solo en lógico, y aceptar el
  de físico —que no las trae— no las borra.
- **Dominios: tipos con nombre, reutilizables.** Se define `Codigo = varchar(500)` una vez
  y las columnas apuntan a él; se cambia el dominio y lo siguen todas, el script incluido.
  El dominio es el dueño del tipo y la columna guarda una copia que se refresca antes de
  que nadie la lea —como `normalizeAuto`—, porque `col.type` lo leen el exportador de SQL,
  el de Prisma, el cálculo de la versión mínima y la tarjeta, y dejar la copia al día
  cuesta una pasada en vez de acordarse del dominio en cada uno de esos sitios. Quitar un
  dominio no cambia el tipo de nadie: las columnas se quedan con el que tienen y solo
  pierden el vínculo, porque ordenar no es tirar trabajo. En lógico la columna se lee
  `Sucursal: Codigo`, que es la única forma de tipo que significa algo en esa vista.
- **La clave primaria lleva su llave**, dibujada y no un emoji: a diez píxeles un emoji
  sale borroso, de un color que no se puede cambiar y distinto en cada sistema. Las filas
  que no son clave llevan el mismo hueco vacío, para que los nombres empiecen todos en la
  misma columna.
- **Lógico y físico: el mismo modelo visto dos veces**, como el conmutador de ERwin. Se
  cambia de vista y el lienzo pone `Pedido de venta · Fecha de emisión` donde ponía
  `dbo.pedido · fecha`. Lo físico sigue siendo la identidad: las relaciones apuntan a
  columnas por su nombre físico y el script se escribe con él, así que un nombre de
  negocio es un rótulo y no puede romper nada —comprobado: escribir uno no mueve ni una
  letra del SQL—. En lógico no se enseñan tipos de motor ni esquemas, porque ninguna de
  las dos cosas existe en un modelo de negocio, y una columna sin nombre lógico se cae al
  físico para que un modelo al que nunca se le pusieron nombres de negocio se vea igual en
  las dos vistas.
- **La vista manda qué nombre se ve Y cuál se escribe.** En lógico el editor escribe el
  nombre lógico y esconde el tipo y el generador; en físico es exactamente el de siempre.
  Si se vieran los dos nombres a la vez habría que explicar cuál es cuál en cada caja.
- **La vista no viaja en el XML**: es de quien mira, no del modelo. Un archivo que se pasa
  a otra persona no debe decidir por ella en qué vista lo abre.
- **La tabla lleva color**, como las entidades de ERwin, para agrupar lo que va junto. El
  modelo guarda el NOMBRE del color y no el píxel: un `#1b5e20` elegido sobre fondo blanco
  es texto negro sobre negro en cuanto se pasa al tema oscuro, así que cada tema tiñe su
  propia superficie con ese tono y se lee en los dos. La cabecera lleva la carga y el
  cuerpo un velo, porque un relleno saturado detrás de quince líneas de texto no se lee en
  ninguno. La lista es corta a propósito: el color sirve para agrupar, y dieciocho verdes
  distintos no agrupan nada. El árbol enseña el mismo punto que la tarjeta.
- **La paleta: se elige el tipo antes de arrastrar**, como en ERwin. Identificadora, no
  identificadora o muchos a muchos. La identificadora mete la clave heredada DENTRO de la
  primaria de la hija —línea continua y esquinas redondeadas— y la otra la deja fuera. El
  tipo no se guarda en ninguna parte: ES dónde están las columnas, así que el rótulo no
  puede decir una cosa mientras el modelo dice otra. Cambiarlo en el diálogo mueve las
  columnas de verdad. La herramienta se usa una vez y vuelve la flecha, para no tener que
  acordarse de desarmarla antes de mover una tabla.
- **El muchos a muchos crea la tabla puente**, que es como se escribe en una base de
  datos: clave sustituta propia y las dos foráneas fuera de la primaria, la forma que
  tienen las demás tablas del diseñador. Marcar PK en las dos columnas la convierte en la
  otra forma, la de primaria compuesta.
- **La clave heredada no pisa la de la hija.** Con la clave sustituta que pone el
  diseñador todas las tablas tienen una columna `id`, y heredar `id` sobre `id` habría
  convertido la propia clave de la hija en foránea; ahora se propone `idRol`, con el
  nombre del padre detrás. Es el «rolename» de ERwin.
- **El modo diseño es un sitio de trabajo, no una página.** Se queda con la ventana
  entera: árbol de objetos a la izquierda —tablas, relaciones y esquemas, cada rama con su
  cuenta—, el lienzo llenando el resto y una barra abajo que dice qué hay seleccionado.
  Es el Model Explorer de ERwin sin las barras grises de 2005. El árbol y el lienzo son
  dos vistas de lo mismo: señalar en una se ve en la otra, un clic selecciona y dos abren.
  El modo comparar sigue siendo la página centrada de siempre, que es lo que le va a un
  informe que se lee de arriba abajo.
- **El editor de la tabla es una rejilla y usa el ancho de la pantalla.** Cada tipo trae
  controles distintos —`varchar` tiene longitud y MAX, `uniqueidentifier` no, `decimal`
  tiene precisión y escala—, así que con filas flexibles cada una salía de un ancho y
  nada quedaba en columna. Con pistas fijas y una cabecera, lo que a una fila no le toca
  deja su hueco y el resto no se mueve: se lee como la tabla que es.
- **Pensado para un modelo de verdad: una columna, una línea.** La entidad escribe cada
  columna como la escribe ERwin —`Nombre: varchar(500) NOT NULL`— y no el nombre a un lado
  peleando con el tipo al otro por el ancho de la caja. Quince columnas —que es lo que
  tiene una tabla de verdad— caben en una caja pequeña y no se recorta ninguna. El orden
  es el del DDL, para que lo que se lee en el diagrama sea lo que va a salir en el script.
- **Las líneas van en ángulo recto**, no en diagonal. Una diagonal entre dos cajas cruza
  por encima de todo lo que haya en medio, y con quince tablas en medio hay siempre algo.
- **La tarjeta se estira también a lo alto.** Con un alto puesto, la lista de columnas se
  desplaza dentro y la clave primaria se queda siempre a la vista: una tabla de treinta
  columnas cabe en la caja que uno le dé en vez de taparlo todo. Doble clic en la esquina
  y vuelve a su tamaño natural. El alto viaja en el XML como el ancho.
- **La entidad del lienzo es una vista, no un formulario**, como en ERwin: la clave
  primaria en el compartimento de arriba separada por una línea, marcas `(FK)`, y esquinas
  redondeadas cuando la entidad depende de otra para identificarse. **Doble clic** abre el
  editor de la tabla, que es donde se escribe y donde las columnas tienen sitio. Las
  líneas llevan **pata de gallo** en el lado «muchos» y van **discontinuas cuando la
  relación no identifica** a la hija. Todo eso se deduce del modelo: quitarle la clave a
  una columna foránea cambia sola la línea y las esquinas.
- **Claves que se rellenan solas:** un contador (`IDENTITY(1,1)`, `AUTO_INCREMENT`,
  `GENERATED BY DEFAULT AS IDENTITY`, `AUTOINCREMENT`) o un identificador único que genera
  el motor (`NEWSEQUENTIALID()`, `gen_random_uuid()`). Cada uno se ofrece solo sobre los
  tipos que ese motor admite de verdad, y la clave de una tabla nueva nace con el tipo
  uuid propio del motor donde lo hay —en SQL Server, `uniqueidentifier`— y con contador
  donde no (MySQL lo guardaría en `char(36)` y SQLite no trae generador). SQLite recibe
  además su trato aparte: el `AUTOINCREMENT` va dentro de la columna, pegado a
  `INTEGER PRIMARY KEY`, que es lo único que ese motor acepta.
- **Versión mínima del motor, calculada y no supuesta:** cada tipo guarda la versión en que
  apareció, se elige una versión objetivo y el diseñador avisa del tipo que es más nuevo
  que ella. El SQL exportado declara la versión que el modelo exige de verdad.
- **El modelo se guarda y se abre en XML**, así que se comparte, entra en git y se lee a
  ojo; y se exporta a **SQL** (SQL Server / PostgreSQL / MySQL / SQLite) y **Prisma**.
- **Conexiones recientes** y **guardar / abrir una comparación** (`.ddcmp`), ambas sin
  guardar contraseñas.
- **Interfaz por fases** (Conexión → Comparación), **bilingüe ES/EN** y con
  **tema claro / oscuro / sistema**.
- **Instalador para macOS** (`.dmg`) generado con `tauri build`.
- **Pruebas:** 109 unitarias en `schemir-core` + 23 en la app: 9 del lector MySQL
  (entrecomillado, valores por defecto, troceo del script), 6 del tipo de fila común, 6 de la
  cadena de conexión ODBC y 2 de integración del lector SQLite. Entre las del núcleo, 12
  cubren el catálogo de tipos: que cada motor traiga los suyos y no los ajenos, que el
  `MAX` sea solo de SQL Server, que la versión mínima salga del tipo más nuevo, y una que
  fija el JSON que la interfaz recibe (renombrar un campo ahí rompe el diseñador sin que
  el compilador diga nada).

## ⚠️ Pendiente de verificar

- **SQL Server ya está verificado de punta a punta contra una base real**, además de SQLite.
  Se leyó un esquema de 151 objetos en uso (25 tablas, 21 vistas, 22 procedimientos, 23
  funciones, 31 foráneas, 30 índices) con autenticación de Windows, en ~1,1 s por los dos
  lados, y el comparador reportó los 17 cambios que se le habían metido a propósito a una
  copia, con el detalle columna por columna. **PostgreSQL y MySQL / MariaDB siguen sin
  verificar**: el código compila y la parte que no necesita servidor está cubierta por tests,
  pero sus consultas a los catálogos no se han ejecutado nunca contra un servidor real.
- **El guardia de pérdida de datos** se comprobó contra una base real en SQLite y en SQL
  Server: el script aborta de verdad y no se aplica nada. En PostgreSQL y en MySQL / MariaDB
  está cubierto solo por tests del SQL generado, no por una ejecución real.
- **Las columnas que se autogeneran**, lo mismo: comprobadas contra una base real en SQL
  Server (el DDL y también la consulta al catálogo — una columna `IDENTITY(100,5)` reporta
  inicio 100 e incremento 5, y agregar una columna `IDENTITY` a una tabla con filas funciona)
  y en SQLite (el `AUTOINCREMENT` numera de verdad, y el lector lo detecta en el DDL de
  `sqlite_master` sin dar falsos positivos). En PostgreSQL y MySQL / MariaDB, solo tests del
  SQL generado.
- **Una comparación mantiene DOS conexiones abiertas a la vez**, una por lado, y no hay
  reintento si alguna se corta a medio leer: se pierde todo el esquema ya leído. Con ODBC el
  caso que lo destapó ya no se da (iba por el DAC, que admite una sola conexión), pero el
  comportamiento sigue ahí para cualquier otro corte.
- **El camino ODBC es más lento que el de TCP:** 3,1 s contra 1,1 s leyendo el mismo esquema de
  151 objetos. Pide cada valor como texto y trae fila por fila. Reservar el buffer de una vez ya
  lo bajó de 12,8 s a 3,1 s —los cuerpos de vistas y procedimientos llegaban a trocitos—, y lo
  que falta es traer las filas por lotes con los buffers columnares de `odbc-api`. No es
  inmediato: esos buffers exigen un tamaño máximo por columna, y un `nvarchar(max)` con el
  cuerpo de un procedimiento se truncaría. Habría que tratar aparte las columnas de definición.
- **Leer la identidad en PostgreSQL exige la versión 10 o posterior**, porque la consulta usa
  `pg_attribute.attidentity`. Contra un servidor 9.x la lectura de tablas fallaría entera;
  no hay por ahora detección de versión.
- En MySQL / MariaDB, tres puntos concretos conviene mirarlos con un servidor delante:
  el `CREATE` de procedimientos y funciones se rearma a partir de `ROUTINES` + `PARAMETERS`
  (no de `SHOW CREATE`); los índices por expresión se leen con una marca en lugar de la
  expresión, porque `information_schema.STATISTICS` deja la columna en NULL; y el
  `AUTO_INCREMENT` ya se lee de `EXTRA`, pero sin servidor delante no se ha visto correr el
  `ALTER … MODIFY COLUMN … AUTO_INCREMENT` que genera Schemir.

## 🔜 Próximo

Las dos primeras son el mismo hueco visto por dos lados: el modelo común guarda de cada
columna nombre, tipo, nulabilidad, valor por defecto, posición, si es parte de la clave
primaria y cómo se autogenera — pero nada más —, y el generador emite por categorías fijas
en vez de ordenar por dependencias.

- **Ordenación por dependencias.** Las unidades salen por categoría (tablas → columnas →
  claves → índices → restricciones → vistas → rutinas → disparadores), no ordenadas por
  quién necesita a quién. Una vista montada sobre otra vista puede salir al revés, y sin
  esto no sirve de nada leer los tipos definidos por usuario: no habría forma de ponerlos
  delante de quien los usa.
- **Mapeo de tipos entre motores.** El tipo se copia tal como lo reportó el origen, así que
  comparar motores *distintos* produce un script que se lee bien y no corre (`nvarchar(50)`
  o `uniqueidentifier` contra PostgreSQL). Comparar el mismo motor contra sí mismo, que es
  el uso real de hoy, no se ve afectado.
- **Columnas calculadas**, que son otra cosa distinta de las que se autogeneran:
  `GENERATED ALWAYS AS (<expresión>)` (y las columnas calculadas de SQL Server) se leen hoy
  como una columna normal del tipo que devuelve la expresión, así que la expresión se pierde.
- **Leer las restricciones `UNIQUE` como restricciones y no como índices únicos.** SQL Server
  las implementa con un índice único y Schemir las lee de `sys.indexes`, así que las emite con
  un `CREATE UNIQUE INDEX` aparte donde SSDT las mete dentro del `CREATE TABLE`. El estado
  final es el mismo, pero una restricción no se suelta con `DROP INDEX` ni se declara sin
  nombre, que es lo que haría falta para no arrastrar al destino el sufijo de relleno del
  origen.
- **El guardia es más miedoso de lo necesario:** considera peligroso cualquier cambio de tipo,
  también los que ensanchan. Pasar de `varchar(400)` a `varchar(800)` no puede perder nada y
  SSDT no lo marca, pero Schemir detiene el script. Falta distinguir ensanchar de estrechar.
- **Más objetos:** secuencias, tipos definidos por usuario, permisos.

## 🚀 Funcionalidad grande

- **Comparar datos de filas en SQL Server, PostgreSQL y MySQL / MariaDB** (hoy solo SQLite).
- **Validación de CA en TLS** (hoy se confía en el certificado del servidor).

## 📦 Pulido y distribución

- **Instaladores** para Windows (`.msi`) y Linux (`.AppImage`) — el `.dmg` de macOS ya está.
- **Capturas** en el README y una versión en inglés.
- **CI** (GitHub Actions) para compilar y correr los tests en cada push.

---

¿Ideas o prioridades distintas? Las sugerencias y contribuciones son bienvenidas. 🙌
