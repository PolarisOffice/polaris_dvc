# Windows PC에서 DVC parity 검증

polaris 엔진의 출력이 업스트림 `hancom-io/dvc`의 `ExampleWindows.exe`
실제 출력과 일치하는지 바이트 레벨로 검증하는 절차. GitHub Actions CI가
계속 실패하던 빌드 이슈들을 전부 스크립트 하나로 자동화해 두었다.

## 선행 준비 (한 번만)

1. **Visual Studio 2022 Community (무료)** 또는 Build Tools 설치
   <https://visualstudio.microsoft.com/downloads>
   → 설치 시 **"Desktop development with C++"** 워크로드 체크 필수
2. **Git for Windows** — <https://git-scm.com/download/win>
3. **Python 3** (jsoncpp amalgamate 단계에 필요) — <https://www.python.org/downloads>
4. **vcpkg** — 없으면 스크립트가 `%TEMP%\polaris-vcpkg\`에 자동 부트스트랩.
   이미 있으면 `VCPKG_ROOT` 환경변수로 가리키면 재사용.

## 실행

리포 루트에서 PowerShell (pwsh 7 권장, Windows PowerShell 5.1도 가능):

```powershell
# 1) 빌드하고 18개 fixture 전부 돌려서 diff만 보여주기
pwsh -File scripts\parity-windows.ps1

# 2) DVC.exe 출력으로 expected.json 을 덮어쓰기 (파리티 앵커 갱신)
pwsh -File scripts\parity-windows.ps1 -WriteExpected

# 3) 일부 케이스만
pwsh -File scripts\parity-windows.ps1 -Only 02_fontsize_mismatch,11_table_border_type_mismatch

# 4) 이미 클론한 dvc 트리를 쓰고 싶을 때
pwsh -File scripts\parity-windows.ps1 -UpstreamPath C:\src\dvc

# 5) 업스트림 특정 ref (브랜치/태그/SHA) 빌드
pwsh -File scripts\parity-windows.ps1 -UpstreamRef 19a985e
```

## 스크립트가 하는 일

1. vcpkg 준비 후 `jsoncpp:x86-windows` 설치
2. `hancom-io/dvc` + OSS 의존성(`hwpx-owpml-model`, `rapidjson`, `jsoncpp`) 클론
3. 업스트림 vcxproj를 in-place 패치
   - OWPML 서브프로젝트의 `<PrecompiledHeader>` → `NotUsing` (v141→v143
     재타겟 이슈 회피)
   - DVCModel.vcxproj의 jsoncpp 경로를 vcpkg 설치 절대경로로 치환
4. `msbuild DVCModel.sln /p:Configuration=Release /p:Platform=x86
   /p:PlatformToolset=v143 /p:WindowsTargetPlatformVersion=<detected>`
5. `testdata\golden\<case>\doc.hwpx`마다 `ExampleWindows.exe -j -a
   --file=<out> -t <spec> <doc>` 실행
6. `-WriteExpected` 없으면 → `testdata\golden\_dvc-output\<case>.json`
   에 결과 쓰고 기존 `expected.json`과 diff. 요약 MATCH/DIFFER/MISSING.
7. `-WriteExpected` 있으면 → DVC.exe 출력으로 `expected.json`을 바로
   덮어쓰기 (parity 앵커로 채택).

## 결과 해석

- **MATCH**: polaris 엔진 출력 == DVC.exe 출력 → 이 케이스는 parity
  달성됨.
- **DIFFER**: polaris와 DVC의 바이트가 다름. `_dvc-output/<case>.json`
  을 보고 엔진을 수정하거나, 필드 처리 로직을 upstream에 맞춤.
- **MISSING**: DVC.exe가 출력 파일을 만들지 않음. 보통 upstream이
  "빈 텍스트" 위반(매크로/스타일만 있고 텍스트 없는 문단)을 실제로
  드롭하는 버그를 그대로 재현한 것. `docs/parity-roadmap.md` 4번 항목
  참고.

## 결과를 리포에 반영하는 플로우 (권장)

```
# Windows에서
pwsh -File scripts\parity-windows.ps1 -WriteExpected

# 바뀐 파일 확인
git diff testdata\golden\*\expected.json

# polaris 엔진이 해당 출력과 일치하도록 수정 (또는 차이를 설명)
# … 코드 수정 …

# 전체 회귀 (Mac/Linux에서도 가능)
cargo test -p polaris-core --test golden
```

## 스크립트가 실패할 때 (알려진 이슈)

- `ExampleWindows.exe`를 실행했는데 JSON 대신 크래시 다이얼로그가 뜨면
  빌드된 `.exe`에 OWPML DLL 경로가 안 맞는 것. `Build/Bin` 디렉터리와
  `ExampleWindows.exe`가 있는 `x86/Release/` 디렉터리에 각각 있는
  OWPML*.dll 파일들을 `.exe` 옆에 복사해 확인.
- `msbuild`가 `C1083 Cannot open include file: 'json/json.h'`로 실패하면
  vcpkg 설치는 됐지만 vcxproj 패치가 안 먹은 것. 스크립트 출력에서
  `patched include/lib paths in DVCModel.vcxproj` 줄이 찍혔는지 확인.
- v143 재타겟 없이 하드 v141을 고집하려면 VS Installer에서 "MSVC v141
  - VS 2017 C++ x64/x86 build tools" 개별 컴포넌트 추가 후 스크립트의
  `/p:PlatformToolset=v143`을 `v141`로 바꾼다.

## 왜 CI가 아니라 로컬인가

GitHub Actions `windows-latest`에서 이 빌드를 한 번 돌리는 데 약 10분이
걸렸고 로그 API가 자주 `BlobNotFound`를 반환해 디버깅 루프가 매우
느렸다. 로컬 Windows에서는 같은 스크립트가 2분 안에 끝난다.
