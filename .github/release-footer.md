
---

## เปิดครั้งแรกบน macOS

ไฟล์ `.dmg` **ไม่ได้เซ็นด้วย Apple Developer certificate** macOS จะบล็อกการเปิด
ครั้งแรกพร้อมข้อความว่า "Apple could not verify Velo is free of malware"
ตัวแอปไม่ได้มีปัญหา macOS แค่ยืนยันไม่ได้ว่าใครเป็นคนบิลด์

1. ลาก **Velo** เข้า Applications แล้วดับเบิลคลิก
2. เมื่อมีข้อความเตือน กด **Done**
3. เปิด **System Settings → Privacy & Security** เลื่อนไปที่หัวข้อ **Security**
   แล้วกด **Open Anyway** ตรงข้อความที่พูดถึง Velo
4. ยืนยันด้วย Touch ID หรือรหัสผ่าน

ทำครั้งเดียวพอ และบน macOS 15 ขึ้นไป วิธีคลิกขวา → Open ใช้ข้ามไม่ได้แล้ว

ถ้าไม่อยากผ่านขั้นตอนนี้ ให้บิลด์เองจากซอร์ส — แอปที่บิลด์ในเครื่องไม่ติด quarantine
