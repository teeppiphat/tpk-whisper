# tpk-whisper

เครื่องมือ dictation (พูดแล้วได้ข้อความ) บน macOS แบบเล็กและเบาที่สุด สไตล์เดียวกับ
MacWhisper — **กดคีย์ลัดค้างไว้แล้วพูด พอปล่อยคี้ ระบบจะถอดเสียงเป็นข้อความด้วย
[Typhoon ASR](https://docs.opentyphoon.ai/en/asr/) (`typhoon-asr-realtime`) แล้ว
แปะข้อความตรงตำแหน่ง cursor ในแอปที่กำลังโฟกัสอยู่ให้อัตโนมัติ**

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
4. ส่งไฟล์ไปที่ `https://api.opentyphoon.ai/v1/audio/transcriptions`
   (รูปแบบเข้ากันได้กับ OpenAI) พร้อมพารามิเตอร์ `model=typhoon-asr-realtime`
   — ตรงกับฟังก์ชัน `transcribe_audio_file` ในเอกสาร Typhoon
5. ข้อความที่ได้ถูกใส่ลง clipboard แล้วสั่ง ⌘V อัตโนมัติเพื่อแปะตรง cursor
6. มีตัวจำกัดอัตราการเรียก (rate limit) ฝั่ง client ที่ **100 ครั้ง/นาที** ตามลิมิตของโมเดล

---

## สิ่งที่ต้องติดตั้งก่อน (บนเครื่อง Mac)

```bash
# 1) Rust (ภาษาและ toolchain หลัก)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2) Xcode command line tools (จำเป็นสำหรับคอมไพล์บน macOS)
xcode-select --install

# 3) Tauri CLI (ตัวสั่ง build/dev)
cargo install tauri-cli --version "^2"
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

1. เปิดแอป — จะไปอยู่บน **menu bar** (มุมขวาบน) ไม่มีไอคอนบน Dock
2. คลิกไอคอนบน menu bar → เลือก **Settings…**
3. วาง **Typhoon API key** (ขอฟรีได้ที่ [playground.opentyphoon.ai](https://playground.opentyphoon.ai/asr))
4. ตั้ง **คีย์ลัด** (ดูหัวข้อถัดไป) แล้วกด **Save**
5. อนุญาตสิทธิ์ของ macOS ตามที่ระบบถาม (ดูหัวข้อ "สิทธิ์ที่ต้องอนุญาต")

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
│   └── index.html        # ฟอร์ม API key + ปุ่มจับคีย์ลัด
└── src-tauri/
    ├── Cargo.toml        # dependencies + โปรไฟล์ release ที่ปรับให้ไฟล์เล็ก
    ├── tauri.conf.json   # ตั้งค่าแอป/หน้าต่าง/bundle
    ├── Info.plist        # ข้อความขอสิทธิ์ไมค์ + LSUIElement (แอป menu bar)
    ├── capabilities/default.json  # สิทธิ์ฝั่ง frontend ของ Tauri
    ├── build.rs
    └── src/
        ├── main.rs       # จุดเริ่มโปรแกรม
        ├── lib.rs        # tray, คีย์ลัด, state, คำสั่ง, pipeline หลัก
        ├── audio.rs      # อัดเสียงด้วย cpal → WAV mono 16-bit
        ├── transcribe.rs # ส่งไฟล์ multipart ไป Typhoon ASR
        ├── paste.rs      # ใส่ clipboard + สั่ง ⌘V ด้วย enigo
        ├── config.rs     # อ่าน/เขียน config (API key, คีย์ลัด) เป็น JSON
        └── ratelimit.rs  # ตัวจำกัด 100 req/นาที แบบ sliding window
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

## ปัญหาที่พบบ่อย (Troubleshooting)

- **build error เรื่องไอคอน** → ยังไม่ได้รัน `cargo tauri icon ...`
- **คีย์ลัดไม่ทำงาน / ไม่แปะข้อความ** → ยังไม่ได้ให้สิทธิ์ Accessibility
- **อัดเสียงแล้วเงียบ / ไม่มีข้อความ** → เช็คสิทธิ์ Microphone และเลือกไมค์ default ให้ถูกใน System Settings → Sound
- **"No API key set"** → ยังไม่ได้ใส่ Typhoon API key ในหน้า Settings
- **"Rate limit reached (100/min)"** → ส่งคำขอเกิน 100 ครั้งใน 1 นาที (ปกติแทบไม่เกิดสำหรับใช้คนเดียว)

---

## License

Apache-2.0 (ตรงกับ license ของโมเดล Typhoon ASR)
