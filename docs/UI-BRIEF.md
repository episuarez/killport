# Killport — Brief de diseño de interfaz (UI)

Documento para encargar el diseño de la interfaz. Describe qué hace la aplicación,
qué datos maneja, y los dos superficies a diseñar: el **menú desplegable de bandeja**
(prioritario, ya funcional) y la **aplicación de escritorio** (ventana, fase siguiente).

---

## 1. Qué es Killport

Utilidad de escritorio para **Windows 10/11** dirigida a desarrolladores. Vive en la
**bandeja del sistema** (system tray, junto al reloj) y responde una pregunta concreta:

> "¿Qué hay escuchando en mis puertos locales ahora mismo, qué lo ha levantado, y cómo
> lo paro o lo reinicio sin abrir la terminal?"

Sustituye el flujo manual `netstat -ano | findstr 3000` → buscar PID → `taskkill /F /PID`
por una vista clara y acciones de un clic.

### Filosofía de producto
- **Dev-first**: por defecto solo muestra procesos de desarrollo (node, python, php,
  bases de datos, docker…). Oculta procesos del sistema operativo y apps de terceros.
- **Ligero y headless**: no abre ninguna ventana al arrancar. Su presencia es el icono
  de bandeja. La ventana de escritorio es opcional/secundaria.
- **Acción inmediata**: ver → entender → actuar (matar, reiniciar, abrir) sin fricción.

### Plataforma técnica (condiciona el diseño)
- Construida en **Rust + Tauri v2**. La UI de ventana, cuando se diseñe, será **web**
  (HTML/CSS/JS dentro de un webview), así que el diseño puede usar lenguaje web moderno.
- El **menú de bandeja es nativo de Windows** (no es web): son menús contextuales del SO.
  Esto **limita** el diseño de esa superficie (ver sección 3).
- Tema: debe soportar **claro y oscuro** siguiendo el sistema.

### Usuario objetivo
Desarrollador (junior a senior) que corre varios servicios locales a la vez:
`npm run dev`, `python -m uvicorn`, `php artisan serve`, contenedores Docker, bases de
datos. Suele tener 2–8 puertos dev activos simultáneamente.

---

## 2. Datos que la app conoce de cada puerto

Esto es lo que el backend ya entrega por cada puerto en escucha. El diseño debe presentar
esta información de forma clara y jerarquizada (no todo tiene el mismo peso).

| Campo | Ejemplo | Peso visual |
|---|---|---|
| **Puerto** | `5173` | Alto — identificador principal |
| **Aplicación** | `Node.js JavaScript Runtime` | Alto — nombre legible del exe |
| **Framework** | `Vite`, `Next.js`, `Django`, `Laravel` | Alto — qué tecnología es |
| **Proyecto** | `demo-app` | Alto — de qué proyecto (carpeta/git) |
| **Tipo (kind)** | `node`, `python`, `php`, `postgresql`, `docker` | Medio — categoría/runtime |
| **Proceso** | `node.exe` | Medio |
| **PID** | `11220` | Bajo — técnico |
| **Origen** | `Servicio: postgresql-x64-16` / `Lanzado por: pwsh.exe` / `ad-hoc` | Medio — ¿es servicio o algo abierto a mano? |
| **Ruta** | `C:\Program Files\nodejs\node.exe` | Bajo — verificación |
| **Contenedor Docker** | `my-postgres` | Medio (si aplica) |
| **Sistema sí/no** | `no` | Bajo — filtro |
| **CPU / RAM** | `0.4%` / `120 MB` | Bajo (disponible, opcional) |
| **URL** | `http://localhost:5173` | Para acciones (abrir/copiar) |
| **Command line** | `node node_modules/.bin/vite` | Bajo — detalle técnico/tooltip |

### Acciones disponibles por puerto
- **Abrir en navegador** (`http://localhost:<puerto>`)
- **Copiar URL**
- **Abrir carpeta del proyecto** (explorador)
- **Abrir en editor** (VS Code u otro configurable)
- **Reiniciar** (mata y relanza con el mismo comando)
- **Matar** (graceful → forzado; mata el árbol de procesos hijos)

### Acciones globales
- **Refrescar** (re-escanea)
- **Arrancar al inicio** (launch at login, on/off)
- **Mostrar procesos del sistema** (toggle, off por defecto)
- **Mostrar procesos sin clasificar** (toggle, off por defecto)
- **Salir**

### Notificaciones (toasts de Windows)
- "Puerto abierto: `:5173 — Vite (node.exe)`" cuando aparece un nuevo puerto dev.
- "Puerto cerrado: `:5173`" cuando desaparece.

---

## 3. Superficie A — Menú desplegable de bandeja (PRIORITARIO)

Es la interfaz principal del MVP y **ya está funcionando**. Se abre con clic
(izquierdo o derecho) sobre el icono de bandeja.

### Restricción importante
Son **menús nativos de Windows**. El diseñador NO puede maquetar libremente esta
superficie como una web: se compone de **ítems de menú** (texto, separadores, submenús,
ítems con checkbox, ítems deshabilitados como "etiquetas" de info). No hay tipografías
custom, ni colores arbitrarios, ni layout en columnas reales. El trabajo de diseño aquí
es de **estructura, jerarquía, redacción y orden de la información**, no de estética
gráfica libre.

### Estructura actual (a mejorar)
Menú raíz = lista de puertos dev. Cada puerto es un **submenú** cuyo título es
`:5173  Node.js JavaScript Runtime`. Al desplegarlo:

```
:5173  Node.js JavaScript Runtime
   ├─ App: Node.js JavaScript Runtime        (línea info, no clicable)
   ├─ Proceso: node.exe (pid 11220)          (línea info)
   ├─ Tipo: node                             (línea info)
   ├─ Origen: lanzado por pwsh.exe           (línea info)
   ├─ Ruta: C:\Program Files\nodejs\...      (línea info, truncada)
   ├─ Sistema: no                            (línea info)
   ├─ Framework: Vite · Proyecto: demo-app   (línea info)
   ├─ ───────────────
   ├─ Abrir en navegador                     (acción)
   ├─ Copiar URL                             (acción)
   ├─ Abrir carpeta                          (acción)
   ├─ Abrir en editor                        (acción)
   ├─ Reiniciar                              (acción)
   └─ Matar (graceful)                       (acción)

─────────────────
☐ Arrancar al inicio
  Refrescar
  Salir
```

### Lo que necesitamos del diseño de esta superficie
1. **Jerarquía y orden óptimos** de las líneas de info: ¿qué va primero, qué se agrupa,
   qué se omite por ruido? El usuario quiere "entenderlo de un vistazo".
2. **Redacción de etiquetas** (labels) clara y consistente (español).
3. **Cómo condensar** sin perder claridad: ¿fusionar líneas? (p. ej. una sola línea
   `node.exe · pid 11220 · ad-hoc`). Idealmente 2–4 líneas de info, no 7.
4. **Título del submenú**: qué dos o tres datos caben para identificar el puerto de un
   vistazo (puerto + framework + proyecto vs puerto + app).
5. **Iconografía nativa**: Windows permite icono por ítem de menú. Proponer si usar
   pequeños iconos (por tipo: node/python/docker/db) y para acciones.
6. **Estado vacío**: cómo comunicar "no hay puertos dev escuchando".
7. **Diseño del icono de bandeja** (16/24/32 px, claro y oscuro): debe leerse a tamaño
   diminuto y comunicar "puertos/desarrollo". Idealmente con **badge de conteo** (nº de
   puertos dev activos) si es viable.

---

## 4. Superficie B — Aplicación de escritorio (ventana)

Opcional en el MVP, pero **es donde el diseño gráfico tiene libertad total** (es web
dentro de Tauri). Se abriría desde el menú de bandeja ("Abrir Killport") o con doble
clic en el icono. Pensada para cuando el usuario quiere una vista amplia y clara, no el
menú comprimido.

### Propósito de la ventana
Una **vista tipo dashboard/tabla** de todos los puertos dev con toda la información bien
presentada, búsqueda/filtro, y acciones por fila — lo que el menú de bandeja no puede
mostrar cómodamente.

### Contenido propuesto (a refinar por el diseñador)
- **Tabla/lista principal**, una fila por puerto, con columnas: Puerto · Aplicación ·
  Framework · Proyecto · Origen (servicio/ad-hoc) · Estado · Acciones.
- **Detalle al seleccionar** una fila: panel lateral o expandible con todo (ruta,
  command line, PID, CPU/RAM, contenedor, padre).
- **Agrupación** opcional por proyecto o por tipo.
- **Búsqueda/filtro** por puerto, proyecto, framework.
- **Toggles** visibles: mostrar sistema / mostrar sin clasificar.
- **Acciones por fila**: abrir, copiar, carpeta, editor, reiniciar, matar.
- **Acción destacada**: "matar" debe ser claro pero con confirmación o estado visual
  (es destructivo).
- **Barra superior**: conteo de puertos, refrescar, ajustes (intervalo de sondeo, editor,
  notificaciones, arranque al inicio).
- **Pantalla de ajustes/preferencias** (puede ser modal o vista): intervalo de polling,
  comando de editor, notificaciones on/off, arranque al inicio, puertos ignorados,
  mostrar sistema/sin clasificar.

### Estados a contemplar
- **Vacío**: sin puertos dev → mensaje útil + sugerencia ("arranca tu dev server").
- **Cargando/refrescando**.
- **Lista poblada** (caso normal, 2–8 filas).
- **Muchos puertos** (>20, p. ej. con sistema visible).
- **Acción en curso** (matando/reiniciando — feedback).
- **Sin permisos** (algún proceso elevado no da datos → mostrar con gracia).

### Tono visual sugerido (orientativo, el diseñador decide)
- Estética de **herramienta de desarrollo**: limpia, densa pero legible, técnica sin ser
  fría. Referencias de tono: paneles de Vercel, Linear, TablePlus, Docker Desktop.
- **Modo claro y oscuro** obligatorios (seguir el sistema).
- Color con función: distinguir tipos (node/python/docker/db) y estados; rojo reservado
  para acción destructiva (matar).
- Densidad de información alta pero jerarquizada — el usuario es técnico.

---

## 5. Entregables esperados del diseño

1. **Icono de bandeja**: set 16/24/32 px, claro y oscuro, con/sin badge de conteo.
2. **Especificación del menú de bandeja**: estructura final de ítems, redacción de
   labels, agrupación, iconos por ítem, estado vacío. (No es maquetación libre — es
   estructura + copy + iconografía sobre menús nativos.)
3. **Ventana de escritorio** (si se aborda): mockups de la vista principal (tabla +
   detalle), estado vacío, ajustes, y los estados de la sección 4. Claro y oscuro.
   Idealmente como diseño web (componentes, tokens de color/tipografía) reutilizable en
   el frontend Tauri.

### Restricciones a respetar
- Windows 10/11, escritorio. No móvil.
- El menú de bandeja es nativo → estructura, no estética libre.
- La ventana es web (Tauri) → libertad gráfica, pero ligera (sin frameworks pesados
  innecesarios; se valorará sistema de componentes simple).
- Soporte claro/oscuro.
- Español en la interfaz.

---

## 6. Resumen de prioridades

1. **Icono de bandeja** (lo primero que se ve).
2. **Estructura/jerarquía del menú desplegable** (interfaz principal del MVP).
3. **Ventana de escritorio** (siguiente fase; aquí está el grueso del diseño gráfico).
