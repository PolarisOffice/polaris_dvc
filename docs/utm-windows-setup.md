# UTM + Windows 11 ARM 로컬 파리티 환경

Apple Silicon Mac 에서 CI 를 거치지 않고 DVC.exe 파리티 검증을 돌리는
가장 빠른 길. 한 번 설정하면 `scripts/parity-windows.ps1` 반복 실행이
~2분 사이클로 가능. 기존 `docs/windows-parity-howto.md` 는 "Windows
환경만 있으면 OK" 가정이고, 이 문서는 그 Windows 환경을 macOS 위에
구축하는 절차.

## 0. 전제

- **Apple Silicon (M1~M5)** — 본 문서는 ARM64 전제
- macOS 13.0+ (Ventura 이상, Virtualization.framework 사용)
- 디스크 여유 100 GB+ (VM 80 GB + ISO 5 GB + 여유)
- 한 시간 정도 시간 (Windows 설치가 대부분)

Intel Mac 은 UTM 에서 Windows 11 x64 ISO 를 쓰면 되지만 성능 페널티
큼. 권장 안 함.

## 1. 필수 소프트웨어 (macOS 측)

```sh
# 이미 설치됐으면 두 줄 스킵
brew install --cask utm          # 무료 하이퍼바이저 GUI
brew install --cask crystalfetch # Windows 11 ARM ISO 자동 다운로더
```

또는 웹에서 직접:
- UTM: <https://mac.getutm.app>
- CrystalFetch: <https://github.com/TuringSoftware/CrystalFetch/releases>

## 2. Windows 11 ARM ISO 준비

**CrystalFetch 사용 (추천)**:

1. CrystalFetch 실행
2. Language: English 또는 Korean, **Edition 은 Home/Pro 아무거나 OK**
   (우리 워크플로에 Pro 전용 기능 없음 — Hyper-V, BitLocker, GPO 등
   전부 사용 안 함. Home 이 설치 크기도 살짝 작음)
3. "Download" → ~5 GB, 속도에 따라 5~20 분
4. 다운 완료하면 `~/Downloads/Windows11_InsiderPreview_Client_ARM64_ko-kr_Vxxx.iso`
   같은 파일 생성

**Microsoft 공식 경로 (대안)**:
- <https://www.microsoft.com/en-us/software-download/windowsinsiderpreviewARM64>
- Insider 프로그램 계정 필요 (무료, Microsoft 계정만 있으면 즉시 가입)

## 3. UTM VM 생성

1. UTM 실행 → "Create a New Virtual Machine" 또는 `+` 버튼
2. **Virtualize** 선택 (Emulate 아님 — ARM-on-ARM 네이티브)
3. **Windows** 선택
4. "Import VHDX Image" 체크 해제, ISO 경로로 2단계에서 받은 `.iso` 지정
5. **Install drivers and SPICE tools** 체크 (가상 네트워크/디스플레이 드라이버 자동 설치됨)
6. 리소스:
   - RAM: **8 GB** 이상 (가능하면 16 GB)
   - CPU: **4 코어** 이상
   - 저장: **80 GB** (빌드 캐시 + Windows 업데이트 여유)
7. Shared Directory:
   - **이 단계에서 reset** → 설치 후 수동 설정이 편함
8. 이름: `Windows 11 parity` 같은 거
9. "Save"

## 4. Windows 설치

VM 실행 → Windows 설치 마법사 진행.

**중요한 팁**:
- 설치 초기에 SHIFT+F10 로 cmd 열어서 `oobe\BypassNRO` 입력 후 Enter
  → Microsoft 계정 요구 우회 가능 (로컬 계정으로 설치)
- 언어는 English-US 권장 (한글로 깔면 VS Build Tools 설치 과정에서
  가끔 엉김). 필요하면 나중에 한국어 디스플레이 팩 추가 설치.

설치 끝나면 데스크톱까지 나오는 데 보통 15~25 분.

## 5. Windows 측 필수 소프트웨어

전부 무료. 관리자 PowerShell 한 번 열어서 **Winget** 으로 일괄 설치:

```powershell
winget install --id Git.Git                              --accept-source-agreements --accept-package-agreements
winget install --id Microsoft.VisualStudio.2022.BuildTools `
    --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --includeRecommended"
winget install --id Python.Python.3.12                   --accept-source-agreements --accept-package-agreements
winget install --id Microsoft.PowerShell                 --accept-source-agreements --accept-package-agreements
winget install --id Kitware.CMake                        --accept-source-agreements --accept-package-agreements
```

VS Build Tools 은 무거움 (~7 GB 다운, 설치 후 ~10 GB). 차 한 잔.

## 6. 저장소 클론 (Windows VM 안에서)

```powershell
# 적당한 작업 디렉토리
cd C:\dev
git clone https://github.com/PolarisOffice/polaris_dvc
cd polaris_dvc
```

## 7. 파리티 스크립트 실행

이미 리포에 `scripts/parity-windows.ps1` 있음. CI 에서 돌리던
**모든 빌드 패치 + 실행** 을 한 번에:

```powershell
# 전체 fixture 대상으로 빌드 + 실행 + diff
pwsh -File scripts\parity-windows.ps1

# 특정 케이스만
pwsh -File scripts\parity-windows.ps1 -Only 01_clean,02_fontsize_mismatch

# DVC.exe 출력으로 expected.json 덮어쓰기 (parity 앵커 갱신)
pwsh -File scripts\parity-windows.ps1 -WriteExpected
```

첫 빌드는 2~4분 (VS 초기화 시간 포함). 이후 증분 빌드는 30초 이내.

## 8. 디버깅 (크래시 조사용)

CI 에서 잡은 크래시 재현·조사가 목적이면:

```powershell
# WinDbg / cdb 설치
winget install --id Microsoft.WindowsSDK.10.0.22621 `
    --override "/features OptionId.WindowsDebugger /quiet"

# dump 생성 자동화 (WER 경유)
reg add 'HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps' `
    /v DumpFolder /t REG_EXPAND_SZ /d 'C:\crashdumps' /f
reg add 'HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps' `
    /v DumpType /t REG_DWORD /d 2 /f

# 실패 명령 실행 (실제 spec 파일 사용 — schemas/jsonFullSpec.json 은
# JSON Schema 레퍼런스라 직접 spec 으로 쓰면 parse 문제. 자세한 구분은
# docs/cli-compat.md 의 "스펙 파일 vs 스키마 파일" 절)
cd C:\dev\polaris_dvc
& "$env:TEMP\polaris-dvc-build\dvc\Release\ExampleWindows.exe" `
    -j -o --file=out.json third_party\dvc-upstream\sample\test.json testdata\real-samples\empty.hwpx

# dump 분석
cdb -z "C:\crashdumps\ExampleWindows.exe.<pid>.dmp"
# cdb> !analyze -v
# cdb> k                     # stack trace
# cdb> lm                    # loaded modules
```

Debug 빌드로 심볼까지 보려면 `parity-windows.ps1` 의 `/p:Configuration=Release`
를 `Debug` 로 바꿔서 재빌드.

## 9. Mac ↔ VM 파일 동기화

세 가지 방법:

**A. UTM Shared Directory (제일 간단)**
UTM 설정 → Sharing → Enable Directory Share → macOS 디렉토리 선택.
VM 안에서 `\\spicevm\<share-name>` 으로 마운트. 단, SPICE 드라이버
설치돼 있어야 함 (Windows 설치 시 "Install drivers and SPICE tools"
체크했으면 자동).

**B. Git 왕복**
가장 깔끔. Windows 에서 수정 → `git push` → Mac 에서 `git pull`.
단순한 스크립트 반복 실행엔 과함.

**C. 네트워크 공유 (SMB)**
macOS 의 System Settings → Sharing → File Sharing 켜고,
VM 안에서 `\\<mac-ip>\<sharename>` 접속.

대부분의 경우 **A** 추천.

## 10. 성능 기대치

M1/M2/M3 기준 측정치 (ARM64 네이티브 Windows 11):

- DVCModel 빌드 (cold): 2~3 분
- DVCModel 빌드 (warm): 10~20 초
- 한 fixture 대상 DVC.exe 실행: < 1 초
- `parity-windows.ps1` 전체 (46 fixture 전부): ~1 분

**CI 의 7~10분 사이클 대비 10배 이상 빠름.**

## 11. VM 세팅 후 흐름 요약

```
Mac 에서:
  코드 수정 (엔진/파서/룰 등)
  git push

Windows VM 에서 (UTM 항상 켜둠):
  git pull
  pwsh -File scripts\parity-windows.ps1 -Only <관심케이스>
  → MATCH/DIFFER 판정 1분 내

Mac 으로:
  DIFFER 케이스만 파고들어 polaris 엔진 재조정
```

## 12. 트러블슈팅

| 증상 | 원인 / 조치 |
|---|---|
| UTM 에서 Windows ISO 부팅 시 "No bootable device" | ISO 가 x64 로 받혔을 가능성. ARM64 ISO 로 재다운 (CrystalFetch 가 자동) |
| VS Build Tools 설치가 "네트워크 오류" | Windows Defender SmartScreen 이 설치 EXE 차단. VM Defender 일시 비활성화 후 재시도 |
| `msbuild` 가 `C1083 json/json.h not found` | `parity-windows.ps1` 의 vcpkg 스텝이 안 돌았거나 패치 실패. 스크립트 출력의 `patched include/lib paths` 라인 확인 |
| `ExampleWindows.exe` 크래시 다이얼로그 | CI 에서 봤던 동일 크래시. §8 의 dump 경로 세팅 후 `cdb` 로 stack 확인 |
| VM 입력 지연 / 화면 끊김 | UTM Settings → Display → Resolution 낮추기, 또는 VM 재시작 |
| 시간대·시계 꼬임 | VM 설정 → System → Sync clock 체크 |

## 13. 이 문서와 겹치는 문서

- `docs/windows-parity-howto.md` — Windows 환경이 이미 있을 때의 실행 가이드
- `docs/dvc-parity-handoff.md` — CI 기반 32+ 실행의 조사 이력과 남은 가설
- `.github/workflows/dvc-parity.yml` — 같은 빌드 레시피의 CI 구현 (참고용)

VM 세팅이 끝나면 본 문서는 덜 볼 일 있고, 실제 작업은
`windows-parity-howto.md` + `dvc-parity-handoff.md` 두 문서 중심으로
진행.
