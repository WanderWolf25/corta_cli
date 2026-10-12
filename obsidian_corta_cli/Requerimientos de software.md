# Especificación de Requisitos de Software (SRS)

## 1. Introducción

### 1.1 Propósito

Definir formalmente los requisitos funcionales, no funcionales, reglas de negocio y arquitectura lógica para **CutEngine CLI**, una herramienta modular de línea de comandos en Rust diseñada para la edición, recorte y ensamblado determinista de video mediante procesamiento de transcripciones, orquestación de FFmpeg y asistencia de modelos de lenguaje (Gemini API).

### 1.2 Alcance del Sistema

CutEngine CLI es un conjunto de herramientas desacopladas que operan bajo el estándar de flujos POSIX (`STDIN`, `STDOUT`, `STDERR`). El sistema no incorpora modelos de machine learning embebidos ni interfaces gráficas. Su función principal es procesar transcripciones generadas por motores externos (ej. Whisper), compilar intenciones en lenguaje natural a directivas matemáticas de corte (`DROP`, `KEEP`), y aplicar un modelo sustractivo sobre una línea de tiempo para renderizar video final mediante stream copy (`-c copy`) con redondeo a keyframes.

## 2. Filosofía Arquitectónica y Principios de Diseño

```
+----------------------------+
|  whisper-ctranslate2 (GPU) |
+----------------------------+
              | (JSON via STDOUT)
              v
       [ transcript.json ]
              |
              | (STDIN)
              v
+----------------------------+
|    cutengine ai-plan       | <--- Directiva: "elimina silencios"
|       (Gemini API)         |
+----------------------------+
              | (STDOUT: comandos DROP/KEEP)
              v
       [ commands.txt ]
              |
              | (STDIN)
              v
+----------------------------+
|     cutengine render       | <--- Video crudo (input.mp4)
|    (Timeline + FFmpeg)     |
+----------------------------+
              |
              v
      [ final_cut.mp4 ]
```

### 2.1 Filosofía UNIX (Plug & Play)

- **Agnosticismo de Componentes:** Cada fase del flujo opera de forma aislada. Si el motor de transcripción (Whisper) o el LLM (Gemini) se actualizan o reemplazan, el núcleo de procesamiento matemático y corte permanece inalterado.
    
- **Separación Estricta de Flujos (POSIX Streams):**
    
    - `STDIN`: Ingesta de datos estructurados (archivos JSON de transcripción o listas de comandos en texto plano).
        
    - `STDOUT`: Emisión exclusiva de datos limpios procesables para el siguiente comando del pipeline.
        
    - `STDERR`: Emisión exclusiva de telemetría, barras de progreso, logs informativos y errores de ejecución.
        
- **Componibilidad:** Todos los subcomandos deben soportar ejecución mediante tuberías (`|`) o redirección de archivos (`<`, `>`).
    

## 3. Modelo Conceptual y Componentes del Sistema

Fragmento de código

```
classDiagram
    class TimeRange {
        +f32 start
        +f32 end
        +validate() Result
        +overlaps(other: TimeRange) bool
    }

    class Timeline {
        +f32 total_duration
        +Vec~TimeRange~ active_segments
        +apply_drop(start: f32, end: f32) Result
        +apply_keep(start: f32, end: f32) Result
        +normalize() void
    }

    class CommandParser {
        +parse_line(raw: str) Result~Command~
        +parse_stream(reader: Read) Result~Vec~Command~~
    }

    class AiCompiler {
        +build_payload(transcript: str, prompt: str) Value
        +compile_to_commands(response: str) Result~Vec~Command~~
    }

    class MediaEngine {
        +probe_duration(path: Path) Result~f32~
        +probe_keyframes(path: Path) Result~Vec~f32~~
        +align_to_keyframe(time: f32, keyframes: &[f32]) f32
        +render_concat(input: Path, ranges: &[TimeRange], output: Path) Result
    }

    Timeline *-- TimeRange : contiene
    CommandParser ..> Timeline : muta
    AiCompiler ..> CommandParser : emite sintaxis
    MediaEngine ..> Timeline : consume segmentos
```

### 3.1 Entidades de Datos

- **`TimeRange`:** Intervalo temporal continuo [start,end] con precisión de coma flotante (`f32`) en segundos absolutos. Invariante: 0≤start<end.
    
- **`TranscriptSegment`:** Registro atómico de transcripción compuesto por [start,end] y una cadena UTF-8 asociada (`text`).
    

### 3.2 Módulos Lógicos

- **`TimelineEngine`:** Máquina de estado matemática que administra los intervalos válidos del video bajo un modelo exclusivamente sustractivo.
    
- **`CommandParser`:** Analizador de flujo que convierte cadenas de texto estructurado en operaciones sobre la línea de tiempo.
    
- **`AiCompiler`:** Cliente de integración que serializa transcripciones a Gemini API y valida esquemas de respuesta JSON para emitir instrucciones canónicas.
    
- **`MediaEngine`:** Adaptador del sistema para interactuar con `ffmpeg` y `ffprobe`, gestionando listas de concatenación y alineación de fotogramas clave.
    

## 4. Requisitos Funcionales (RF)

### 4.1 Ingesta Multimedia y Extracción Externa (RF01)

- **RF01.1:** El subcomando de renderizado debe validar contenedores multimedia estándar: `.mp4`, `.mkv` y `.mov`.
    
- **RF01.2:** El sistema debe delegar la extracción de audio a herramientas externas o subcomandos dedicados, asegurando formato PCM sin comprimir (`16-bit`, `16 kHz`, `mono`, `.wav`) para compatibilidad directa con Whisper.
    

### 4.2 Transcripción Desacoplada (RF02)

- **RF02.1:** El sistema debe consumir esquemas JSON de transcripción que contengan arrays de objetos con campos mínimos:
    
    - `start`: Tiempo inicial en segundos (`f32`).
        
    - `end`: Tiempo final en segundos (`f32`).
        
    - `text`: Texto transcrito en UTF-8.
        
- **RF02.2:** La ingesta de dicho JSON debe funcionar indistintamente a través de un archivo local especificado por argumento (`--transcript <path>`) o por medio del descriptor de entrada estándar (`STDIN`).
    

### 4.3 Motor de Línea de Tiempo Sustractiva (RF03)

- **RF03.1 (Estado Inicial Obligatorio):** La línea de tiempo debe inicializarse en **Modo Sustractivo Único**, cubriendo el intervalo completo del video original:
    
    Timeline0​={[0.0,Ttotal​]}
    
- **RF03.2 (Semántica DROP):**
    
    - La instrucción `DROP <start> - <end>` resta el intervalo del conjunto de segmentos activos.
        
    - Si el corte se ubica en el interior de un segmento activo, este se bifurca en dos subsegmentos contiguos.
        
    - Si el corte colisiona con un extremo, el segmento se trunca.
        
    - Si el corte cubre completamente uno o más segmentos, se eliminan de la colección.
        
- **RF03.3 (Semántica KEEP):**
    
    - La instrucción `KEEP <start> - <end>` reincorpora o asegura la inclusión del intervalo dentro del conjunto activo.
        
- **RF03.4 (Normalización y Ordenamiento Cronológico):**
    
    - Tras cada operación o antes del renderizado, los segmentos deben ordenarse ascendentemente según su tiempo de inicio (`start`).
        
    - Cualquier par de segmentos contiguos cuyo desfase temporal sea menor o igual a ϵ=0.05 s debe unificarse automáticamente para evitar microcortes en el contenedor.
        

### 4.4 Parser de Comandos Crudos (RF04)

- **RF04.1:** El analizador debe interpretar tres formatos temporales válidos:
    
    - Absoluto: `HH:MM:SS.mmm` (ej. `00:01:23.500`).
        
    - Simplificado: `MM:SS` o `MM:SS.mmm` (ej. `01:23.5`).
        
    - Segundos puros: `SSSS.ss` (ej. `83.5`).
        
- **RF04.2:** Soporte de comandos línea por línea desde archivo o `STDIN`:
    
    - `DROP <start> - <end>`
        
    - `KEEP <start> - <end>`
        
    - `STATUS` (emite a `STDERR` la duración final acumulada y los bloques conservados).
        
    - `RESET` (restaura el intervalo inicial [0.0,Ttotal​]).
        
- **RF04.3:** Líneas vacías y líneas con prefijo `#` deben ser tratadas como comentarios e ignoradas.
    

### 4.5 Compilador de Directivas de IA (RF05)

- **RF05.1:** El subcomando `ai-plan` debe requerir un prompt en lenguaje natural (ej. `"elimina silencios y partes donde me trabo"`).
    
- **RF05.2:** El payload enviado a Gemini API debe contener la transcripción estructurada y forzar un esquema de respuesta JSON estricto:
    
    JSON
    
    ```
    [
      {
        "action": "DROP",
        "start": 12.5,
        "end": 18.2,
        "reason": "silencio prolongado"
      }
    ]
    ```
    
- **RF05.3:** La salida del compilador debe emitirse directamente a `STDOUT` como una secuencia de comandos crudos legibles:
    
    Plaintext
    
    ```
    DROP 00:00:12.500 - 00:00:18.200
    ```
    
- **RF05.4:** Intervalos devueltos que violen los límites del video (t<0 o t>Ttotal​) deben ser descartados con advertencia a `STDERR`.
    

### 4.6 Ensamble Multimedia y Gestión de Keyframes (RF06)

- **RF06.1 (Fast Copy sin Transcodificación):** La exportación debe realizarse usando el motor de copia de flujos de FFmpeg (`-c:v copy -c:a copy`) mediante el _concat demuxer_.
    
- **RF06.2 (Redondeo a Keyframes):** Para garantizar cortes limpios sin decodificación completa, los puntos de inicio y fin deben ajustarse matemáticamente al keyframe (I-frame / IDR) más próximo calculado previamente con `ffprobe`.
    
- **RF06.3 (Limpieza Determinista):** Los archivos temporales generados (listas de concatenación, segmentos transitorios) deben eliminarse automáticamente tras la ejecución exitosa o interrupción del proceso (`SIGINT`/`SIGTERM`).
    

## 5. Requisitos No Funcionales (RNF)

|Identificador|Categoría|Especificación|
|---|---|---|
|**RNF01**|**Rendimiento**|La normalización matemática y resolución de cortes sustractivos sobre al menos 500 segmentos en memoria debe ejecutarse en un tiempo inferior a 10 ms.|
|**RNF02**|**Concurrencia**|Las operaciones de red (Gemini API) y ejecución de procesos secundarios (`ffmpeg`, `ffprobe`) deben gestionarse de forma asíncrona mediante `tokio`.|
|**RNF03**|**Seguridad**|La clave `GEMINI_API_KEY` debe leerse exclusivamente del entorno del sistema operativo. Queda prohibida su exposición en argumentos CLI o volcados de logs.|
|**RNF04**|**Cumplimiento POSIX**|La salida de datos a `STDOUT` debe ser limpia y canalizable. Errores, advertencias y reportes deben dirigirse obligatoriamente a `STDERR`. En caso de falla, el código de salida debe ser ≥1.|
|**RNF05**|**Portabilidad**|El código debe compilar con el toolchain estable de Rust (edición 2021 o superior) sin warnings, soportando arquitecturas x86_64 en Linux y Windows.|

## 6. Casos de Prueba Críticos y Validación Matemática

### Caso 1: DROP Interno (Bifurcación)

- **Estado inicial:** {[0.0,100.0]}
    
- **Entrada:** `DROP 00:00:20 - 00:00:30`
    
- **Salida requerida:** {[0.0,20.0],[30.0,100.0]}
    

### Caso 2: DROP Superpuesto

- **Estado inicial:** {[0.0,20.0],[30.0,100.0]}
    
- **Entrada:** `DROP 00:00:15 - 00:00:40`
    
- **Salida requerida:** {[0.0,15.0],[40.0,100.0]}
    

### Caso 3: Redondeo de Keyframe

- **Corte solicitado:** start=14.32 s
    
- **Keyframes disponibles:** [12.00,14.50,17.00]
    
- **Resultado aplicado:** start=14.50 s (evita fotogramas corruptos en stream copy).