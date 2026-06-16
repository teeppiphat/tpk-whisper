# tpk-whisper

เครื่องมือ dictation (พูดแล้วได้ข้อความ) บน macOS แบบเล็กและเบาที่สุด สไตล์เดียวกับ
MacWhisper — **กดคีย์ลัดค้างไว้แล้วพูด พอปล่อยคีย์ ระบบจะถอดเสียงเป็นข้อความด้วย
[Typhoon ASR](https://github.com/scb-10x/typhoon-asr) (`typhoon-asr-realtime`) แล้ว
แปะข้อความตรงตำแหน่ง cursor ในแอปที่กำลังโฟกัสอยู่ให้อัตโนมัติ**

ค่าเริ่มต้นรันโมเดล **บนเครื่อง (Local, offline)** ไม่ต้องใช้ API key — หรือจะสลับไป
ใช้ Typhoon API (cloud) ในหน้า Settings ก็ได้

ตัวแอปเป็น Tauri v2 + Rust รันอยู่บน **menu bar** อย่างเดียว ไม่มีไอคอนบน Dock
ไม่มีหน้าต่างหลัก ใช้ WebView ของระบบ (WKWebView) ไม่แบก Chromium เหมือน Electron
หน้า Settings เป็น HTML ล้วน ไม่ต้องใช้ npm/บันเดิลใด ๆ

> รายละเอียดสถาปัตยกรรมเชิงลึกอ่านได้ที่ [`ARCHITECTURE.md`](./ARCHITECTURE.md)
> README ภาษาอังกฤษอยู่ที่ [`README.md`](./README.md)

---

## การทำงานโดยสรุป

1. **กดคีย์ลัดค้าง** (ค่าเริ่มต้น `Ctrl+Alt+D`) → เริ่มอัดเสียงจากไมโครโฟน (push-to-talk)
2. **ปล่อยคีย์** → หยุดอัด
3. เสียงถูกอัดด้วย `cpal` แปลงเป็น mono 16-bit เขียนเป็นไฟล์ `.wav` ชั่วคราว
4. ถอดเสียงด้วย backend ที่เลือกไว้:
   - **Local (ค่าเริ่มต้น):** รันสคริปต์ `local_transcribe.py` ที่ฝังมากับแอป เรียก
     `typhoon-asr` บนเครื่อง ผ่าน launcher ที่ตั้งไว้ (ค่าเริ่มต้นใช้ uv)
   - **API:** ส่งไฟล์ไปที่ `https://api.opentyphoon.ai/v1/audio/transcriptions`
     (รูปแบบเข้ากันได้กับ OpenAI) พร้อม `model=typhoon-asr-realtime`
5. ข้อความที่ได้ถูกใส่ลง clipboard แล้วสั่ง ⌘V อัตโนมัติเพื่อแปะตรง cursor
6. ไฟล์ WAV ชั่วคราวถูกลบทันทีหลังถอดเสียง ถ้ามีค้าง (เช่นแอป crash) จะถูกกวาดทิ้งตอนเปิดแอปครั้งถัดไป

> โหมด API มี rate limit ฝั่ง client ที่ **100 ครั้ง/นาที** ตามลิมิตของโมเดล;
> โหมด Local ไม่มี key ไม่ต่อเน็ต และไม่มี rate limit

---

## สิ่งที่ต้องติดตั้งก่อน (บนเครื่อง Mac)

```bash
# 1) Rust (ภาษาและ toolchain หลัก)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2) Xcode command line tools (จำเป็นสำหรับคอมไพล์บน macOS)
xcode-select --install

# 3) Tauri CLI (ตัวสั่ง build/dev)
cargo install tauri-cli --version "^2"

# 4) uv (ใช้โดย local backend ที่เป็นค่าเริ่มต้น)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

> ต้องใช้ Rust เวอร์ชัน 1.77 ขึ้นไป และ macOS 11 (Big Sur) ขึ้นไป

---

## สร้างไอคอนแอป (ทำครั้งเดียว)

Tauri ต้องมีชุดไอคอนตามที่อ้างถึงใน `tauri.conf.json` สร้างจากรูป PNG สี่เหลี่ยมจัตุรัส
รูปไหนก็ได้ (แนะนำ 1024×1024):

```bash
cd tpk-whisper
cargo tauri icon path/to/your-logo.png
```

คำสั่งนี้จะสร้างไฟล์ลงในโฟลเดอร์ `src-tauri/icons/` ให้เอง
**ถ้ายังไม่ทำขั้นตอนนี้ ตอน build จะ error ว่าหาไอคอนไม่เจอ**

---

## รัน / Build

```bash
cd tpk-whisper

# โหมดพัฒนา (เปิดแอปขึ้น menu bar)
cargo tauri dev

# build ตัวจริง → ได้ไฟล์ที่
# src-tauri/target/release/bundle/macos/tpk-whisper.app
cargo tauri build
```

---

## การตั้งค่าครั้งแรก

ค่าเริ่มต้นของแอปคือ **โหมด Local (offline)** ที่รันผ่าน `uv` ให้อัตโนมัติ —
ขอแค่มี [`uv`](https://docs.astral.sh/uv/) อยู่ในเครื่อง ไม่ต้องใส่ API key
ไม่ต้อง `pip install` เอง

1. ติดตั้ง `uv` ถ้ายังไม่มี: `curl -LsSf https://astral.sh/uv/install.sh | sh`
2. เปิดแอป — จะไปอยู่บน **menu bar** (มุมขวาบน) ไม่มีไอคอนบน Dock
3. คลิกไอคอนบน menu bar → เลือก **Settings…**
4. ตั้ง **คีย์ลัด** (ดูหัวข้อถัดไป) แล้วกด **Save**
5. อนุญาตสิทธิ์ของ macOS ตามที่ระบบถาม (ดูหัวข้อ "สิทธิ์ที่ต้องอนุญาต")

> ครั้งแรกที่กดถอดเสียง `uv` จะดึง Python 3.10 + `typhoon-asr` (torch/NeMo หลาย GB)
> มาเตรียมไว้ อาจใช้เวลาสักครู่ ครั้งต่อ ๆ ไปจะเร็วเพราะ cache แล้ว
>
> ถ้าอยากใช้ **Typhoon API (cloud)** แทน: ไปที่ Settings → เปลี่ยน backend เป็น
> *Typhoon API* แล้ววาง API key (ขอฟรีที่ [playground.opentyphoon.ai](https://playground.opentyphoon.ai/settings/api-key))

---

## การตั้งคีย์ลัด (กดคีย์เองได้เลย)

ในหน้า Settings:

1. คลิกปุ่ม **Record**
2. **กดคีย์ลัดที่ต้องการ** เช่น กด ⌃ (Control) + ⌥ (Option) ค้างไว้แล้วเคาะ D
3. ช่องคีย์ลัดจะเปลี่ยนเป็นค่าที่จับได้โดยอัตโนมัติ (เช่น `Control+Alt+KeyD`)
4. กด **Save** เพื่อให้มีผลทันที (แอป re-register คีย์ลัดใหม่ให้เลย)
5. กด **Esc** ระหว่างกำลังจับคีย์เพื่อยกเลิก

ข้อกำหนด:

- ต้องมีปุ่ม modifier อย่างน้อย 1 ปุ่ม (Control / Alt / Shift / Super) ผสมกับปุ่มหลัก
  เพื่อกันไม่ให้คีย์ลัดไปชนกับการพิมพ์ปกติ
- `Super` คือปุ่ม ⌘ (Command) บน Mac
- ตัวอักษรจะถูกเก็บในรูปแบบ `KeyX` (เช่น D = `KeyD`) ตาม syntax ของ Tauri

**วิธีใช้งานหลังตั้งค่า:** *กดคีย์ลัดค้าง* = กำลังอัดเสียง, *ปล่อยคีย์* = หยุดอัดแล้วถอดเสียงทันที
(push-to-talk)

---

## สิทธิ์ที่ต้องอนุญาต (macOS)

ไปที่ **System Settings → Privacy & Security** แล้วเปิดสิทธิ์เหล่านี้:

| สิทธิ์ | ใช้ทำอะไร |
|--------|-----------|
| **Microphone** | อัดเสียงพูด (ระบบจะเด้งถามให้อัตโนมัติครั้งแรกที่อัด) |
| **Accessibility** | จำเป็นมาก — เพื่อให้คีย์ลัดทำงานได้แม้โฟกัสอยู่ที่แอปอื่น และเพื่อให้แอปสั่ง ⌘V แปะข้อความได้ |
| **Input Monitoring** | บางกรณีระบบขอเพิ่มเพื่อดักการกดคีย์ทั่วระบบ |

> ถ้าคีย์ลัดไม่ทำงาน หรือถอดเสียงได้แต่ไม่แปะข้อความ — เกือบทุกครั้งเกิดจาก
> **ยังไม่ได้ให้สิทธิ์ Accessibility** ลองปิด/เปิดสิทธิ์ของ `tpk-whisper` ในรายการนั้น
> แล้วเปิดแอปใหม่
>
> หมายเหตุ: ตอนรันด้วย `cargo tauri dev` ตัวที่ต้องได้สิทธิ์ Accessibility คือ
> **โปรแกรม terminal** ที่คุณใช้สั่งรัน (ไม่ใช่ตัวแอป) เพราะแอปทำงานในฐานะลูกของ terminal

---

## โครงสร้างโปรเจกต์

```
tpk-whisper/
├── ARCHITECTURE.md       # อธิบายดีไซน์ + ไดอะแกรม flow
├── README.md             # README ภาษาอังกฤษ
├── README.th.md          # ไฟล์นี้
├── src/                  # หน้า Settings (HTML ล้วน ไม่มี bundler)
│   └── index.html        # ฟอร์ม API key + ปุ่มจับคีย์ลัด + เลือก backend
└── src-tauri/
    ├── python/
    │   └── local_transcribe.py  # สคริปต์รันโมเดล local (ฝังในไบนารีด้วย include_str!)
    ├── Cargo.toml        # dependencies + โปรไฟล์ release ที่ปรับให้ไฟล์เล็ก
    ├── tauri.conf.json   # ตั้งค่าแอป/หน้าต่าง/bundle
    ├── Info.plist        # ข้อความขอสิทธิ์ไมค์ + LSUIElement (แอป menu bar)
    ├── capabilities/default.json  # สิทธิ์ฝั่ง frontend ของ Tauri
    ├── build.rs
    └── src/
        ├── main.rs       # จุดเริ่มโปรแกรม
        ├── lib.rs        # tray, คีย์ลัด, state, คำสั่ง, pipeline, กวาดไฟล์ temp
        ├── audio.rs      # อัดเสียงด้วย cpal → WAV mono 16-bit
        ├── transcribe.rs # backend ทั้งสอง: Typhoon API + subprocess รัน local
        ├── paste.rs      # ใส่ clipboard + สั่ง ⌘V ด้วย enigo (ใช้ raw keycode ปุ่ม V)
        ├── config.rs     # อ่าน/เขียน config (backend, key, คีย์ลัด, launcher, …) เป็น JSON
        └── ratelimit.rs  # ตัวจำกัด 100 req/นาที แบบ sliding window (เฉพาะโหมด API)
```

ไฟล์ config จะถูกเก็บที่
`~/Library/Application Support/ai.bedrock.tpkwhisper/config.json`

---

## ทำไมถึง "เบา"

- **Tauri v2** ได้ binary เนทีฟไฟล์เดียว ใช้ WebView ของระบบ ไม่แบก Chromium
- หน้า Settings เป็น HTML/JS ล้วน — ไม่มี React, ไม่มีขั้นตอน build frontend
- **cpal** เป็น binding บาง ๆ ครอบ CoreAudio, **hound** เป็น encoder WAV ขนาดเล็ก
- โปรไฟล์ release ตั้ง `opt-level="s"` + LTO + strip เพื่อบีบขนาดไฟล์
- ตั้ง `LSUIElement` ให้เป็นแอป menu bar ล้วน ไม่กินพื้นที่ Dock

---

## โหมด Local (offline) — ค่าเริ่มต้น

แอปตั้งค่าเริ่มต้นให้รันโมเดล `typhoon-asr-realtime` **บนเครื่องตัวเอง** ผ่าน `uv`
ไม่ต้องใช้ API key ไม่ต้องต่อเน็ต และข้อมูลเสียงไม่ออกนอกเครื่อง
(ใช้แพ็กเกจ [`typhoon-asr`](https://github.com/scb-10x/typhoon-asr) ของ SCB10X ผ่านสคริปต์ Python เล็ก ๆ ที่ฝังมากับแอป)

ค่า launcher เริ่มต้นคือ `uv run --python 3.10 --with typhoon-asr python` ซึ่ง `uv`
จะเตรียม Python 3.10 + แพ็กเกจให้เองอัตโนมัติ — **ไม่ต้องติดตั้งอะไรเพิ่มนอกจากตัว `uv`**
ส่วนวิธีอื่น (pip / venv) ด้านล่างเป็นทางเลือกถ้าไม่อยากใช้ uv

**ติดตั้ง** (ต้องมี Python 3.10) — เลือกวิธีใดวิธีหนึ่ง:

วิธี pip ปกติ:

```bash
pip install typhoon-asr
```

### ใช้กับ uv (แนะนำสำหรับเครื่องนี้)

มี 2 แนวทาง:

**แนวทาง A — ไม่ต้องติดตั้งล่วงหน้า (ง่ายสุด):** ปล่อยให้ `uv run` จัดการ env เอง
ในหน้า Settings ช่อง **Python interpreter / launcher** ใส่:

```
uv run --with typhoon-asr python
```

แอปจะแยกคำสั่งด้วยช่องว่างเอง แล้วรันเป็น
`uv run --with typhoon-asr python local_transcribe.py <wav> --model … --device …`
— uv จะสร้าง environment ชั่วคราว (cache ไว้ ครั้งต่อไปเร็วขึ้น) พร้อม `typhoon-asr` ให้อัตโนมัติ
ไม่ต้อง `pip install` เองเลย ขอแค่มี `uv` อยู่ใน PATH

**แนวทาง B — สร้าง venv ถาวรด้วย uv** (ควบคุมเวอร์ชันได้ชัดเจน):

```bash
cd ~/.tpk-whisper-asr        # โฟลเดอร์ไหนก็ได้
uv venv --python 3.10
uv pip install typhoon-asr
echo "$PWD/.venv/bin/python" # ได้ path ของ interpreter
```

แล้วเอา path ที่ได้ (เช่น `/Users/teeppiphatp/.tpk-whisper-asr/.venv/bin/python`)
ไปใส่ในช่อง **Python interpreter / launcher**

> เบื้องหลังใช้ NVIDIA NeMo + PyTorch + librosa + soundfile ขนาดรวมหลาย GB
> ครั้งแรกที่ใช้จะดาวน์โหลดน้ำหนักโมเดล (~114M params) จาก HuggingFace อัตโนมัติ
> รันบน CPU ได้ (เร็วกว่า real-time, RTF ~0.3x) หรือใช้ GPU ถ้ามี CUDA

**วิธีใช้:** เปิด Settings → ช่อง **Transcription backend** เลือก **Local model (offline)**
แล้วตั้งค่า:

- **Python interpreter / launcher** — ใส่ได้ทั้ง path ของ python (เช่น `python3`,
  `/path/.venv/bin/python`) หรือคำสั่ง launcher เต็ม ๆ เช่น `uv run --with typhoon-asr python`
  (ดูหัวข้อ "ใช้กับ uv" ด้านบน) แอปจะแยก argument ด้วยช่องว่างให้เอง
- **Device** — `auto` / `cpu` / `cuda` (Mac ทั่วไปใช้ `cpu`)
- **Model id** — ค่าเริ่มต้น `scb10x/typhoon-asr-realtime` (เปลี่ยนเป็นรุ่นอีสาน
  `scb10x/typhoon-isan-asr-realtime` ได้)

กด **Save** แล้วใช้คีย์ลัดเหมือนเดิม — ขั้นตอนกด/ปล่อยคีย์, การแปะข้อความ ฯลฯ
เหมือนกันทุกอย่าง ต่างแค่การถอดเสียงเกิดบนเครื่องแทนการเรียก API
(โหมด Local จะข้ามการเช็ค API key และ rate limit ให้อัตโนมัติ)

> แอปจะต่อ `PATH` ให้ child process ครอบ `~/.local/bin`, `~/.cargo/bin`,
> `/opt/homebrew/bin` ฯลฯ ให้เอง เพื่อให้หา `uv`/`python` เจอแม้เปิดแอปจาก Finder
> (ไม่ใช่ผ่าน terminal)

**API กับ Local เทียบกัน:**

| | API (cloud) | Local (offline) |
|---|---|---|
| ต้องมี API key | ใช่ | ไม่ |
| ต้องต่อเน็ต | ใช่ | เฉพาะตอนโหลดโมเดลครั้งแรก |
| ความเป็นส่วนตัว | เสียงถูกส่งขึ้น cloud | ไม่ออกนอกเครื่อง |
| ติดตั้งเพิ่ม | ไม่ต้อง | ต้องลง Python + `typhoon-asr` (หลาย GB) |
| ความเร็ว | เร็ว (ขึ้นกับเน็ต) | ขึ้นกับ CPU/GPU |
| Rate limit | 100 req/นาที | ไม่มี |

---

## หมายเหตุ / ข้อจำกัด

- **โหลดโมเดลใหม่ทุกครั้ง:** ตอนนี้โหมด Local โหลดโมเดลใหม่ทุกครั้งที่ถอดเสียง
  เลยมีดีเลย์ไม่กี่วินาทีก่อนข้อความออก (แก้ได้ด้วยการทำ persistent worker ที่โหลด
  โมเดลค้างไว้ครั้งเดียว — ดู ARCHITECTURE.md)
- **การแปะข้อความ:** ใช้การจำลอง ⌘V กับ clipboard ดังนั้นจะเขียนทับ clipboard เดิม
  ชั่วขณะ และจะไม่ทำงานในแอปที่บล็อก synthetic input
- **โมเดลเน้นภาษาไทย:** `typhoon-asr-realtime` ออปติไมซ์มาเพื่อภาษาไทยเป็นหลัก

---

## ปัญหาที่พบบ่อย (Troubleshooting)

- **build error เรื่องไอคอน** → ยังไม่ได้รัน `cargo tauri icon ...`
- **คีย์ลัดไม่ทำงาน / ไม่แปะข้อความ** → ยังไม่ได้ให้สิทธิ์ Accessibility
- **อัดเสียงแล้วเงียบ / ไม่มีข้อความ** → เช็คสิทธิ์ Microphone และเลือกไมค์ default ให้ถูกใน System Settings → Sound
- **"No API key set"** → ยังไม่ได้ใส่ Typhoon API key ในหน้า Settings
- **"Rate limit reached (100/min)"** → ส่งคำขอเกิน 100 ครั้งใน 1 นาที (ปกติแทบไม่เกิดสำหรับใช้คนเดียว)
- **โหมด Local: "typhoon-asr not installed"** → ยังไม่ได้ `pip install typhoon-asr` ใน python ที่ระบุ
- **โหมด Local: "could not launch python3"** → path ของ Python ไม่ถูก ใส่ path เต็มของ interpreter ที่ลงแพ็กเกจไว้
- **โหมด Local ช้าครั้งแรก** → ปกติ เพราะกำลังโหลดโมเดลจาก HuggingFace ครั้งถัดไปจะเร็วขึ้น

---

## License

Apache-2.0 (ตรงกับ license ของโมเดล Typhoon ASR)
