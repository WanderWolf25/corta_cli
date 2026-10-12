import json
import subprocess
import os
import wave
import numpy as np
from faster_whisper import WhisperModel

video_input = "sample_input.mp4"
temp_audio = "temp_audio.wav"

# 1. Extraer audio como WAV mono a 16kHz PCM con FFmpeg
print("Extracting audio with ffmpeg...")
res = subprocess.run(
    ["ffmpeg", "-y", "-i", video_input, "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", temp_audio],
    capture_output=True,
    text=True,
)

if res.returncode != 0:
    print(f"FFmpeg error: {res.stderr}")
    exit(1)

# 2. Leer audio crudo sin pasar por PyAV
print("Reading PCM samples...")
with wave.open(temp_audio, "rb") as wf:
    frames = wf.readframes(wf.getnframes())
    # Convertir muestras de 16-bit PCM entero (-32768 a 32767) a float32 (-1.0 a 1.0)
    audio_data = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0

# 3. Cargar el modelo
print("Loading Whisper model...")
model = WhisperModel("small", device="cpu", compute_type="int8")

# 4. Transcribir pasando el buffer de NumPy directamente
print("Transcribing...")
segments, info = model.transcribe(
    audio_data,
    language="es",
    condition_on_previous_text=False,
    vad_filter=True,
)

data = {
    "duration": info.duration,
    "segments": [
        {"start": round(s.start, 3), "end": round(s.end, 3), "text": s.text.strip()}
        for s in segments
    ],
}

with open("sample_input.json", "w", encoding="utf-8") as f:
    json.dump(data, f, ensure_ascii=False, indent=2)

if os.path.exists(temp_audio):
    os.remove(temp_audio)

print("Transcription finished successfully: sample_input.json generated.")