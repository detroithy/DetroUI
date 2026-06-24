const koffi = require('koffi');

const user32 = koffi.load('user32.dll');

const FindWindowW = user32.func('FindWindowW', 'void*', ['str16', 'str16']);
const FindWindowExW = user32.func('FindWindowExW', 'void*', ['void*', 'void*', 'str16', 'str16']);
const SetParent = user32.func('SetParent', 'void*', ['void*', 'void*']);
const ShowWindow = user32.func('ShowWindow', 'bool', ['void*', 'int']);
const SendMessageW = user32.func('SendMessageW', 'int64', ['void*', 'uint32', 'int64', 'int64']);
const SetWindowPos = user32.func('SetWindowPos', 'bool', ['void*', 'void*', 'int', 'int', 'int', 'int', 'uint32']);

function embedBehindDesktop(hwndBuffer) {
  try {
    // Buffer'dan pointer'a cevir
    const hwndPtr = koffi.decode(hwndBuffer, 0, 'void*');

    const progman = FindWindowW('Progman', null);
    if (!progman) { console.error('Progman not found'); return false; }

    // WorkerW olustur
    SendMessageW(progman, 0x052C, 0, 1);

    // Progman'in altindaki WorkerW'lari tara
    let workerW = null;
    let prev = null;
    for (let i = 0; i < 20; i++) {
      const w = FindWindowExW(null, prev, 'WorkerW', null);
      if (!w) break;
      workerW = w;
      prev = w;
    }

    if (!workerW) {
      workerW = progman;
    }

    // Pencereyi masaustunun altina yerlestir
    SetParent(hwndPtr, workerW);

    // Tam ekran yap
    const SWP_NOACTIVATE = 0x0010;
    const SWP_SHOWWINDOW = 0x0040;
    const HWND_BOTTOM = 1;
    SetWindowPos(hwndPtr, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOACTIVATE | SWP_SHOWWINDOW);

    return true;
  } catch (e) {
    console.error('embedBehindDesktop error:', e);
    return false;
  }
}

module.exports = { embedBehindDesktop };
