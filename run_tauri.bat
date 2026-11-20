@echo off
echo Starting Isle (Tauri Dev Mode)...
python -c "import os,struct;p=r'src-tauri\target\debug\app.exe';data=bytearray(open(p,'rb').read());pe=struct.unpack('<I',data[0x3c:0x40])[0];va=struct.unpack('<I',data[pe+24+112+16:pe+24+112+20])[0];diff=va-0x22d5000;[data.__setitem__(slice(i,i+4),struct.pack('<I',rva+diff)) for rva in [0x22d5118,0x22d5b80,0x22d5b94,0x22d5ba8] for i in range(len(data)-4) if data[i:i+4]==struct.pack('<I',rva)];open(p,'wb').write(data)"
start "" "src-tauri\target\debug\app.exe"
