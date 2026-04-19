# HWPX 활용 예제 : 문서 적합성 검증 (HWPX Document Validation Checker : DVC)


## 개요
+ HWPX 문서의 내용이 정해진 형태로 작성이 되었는지를 검증하는 프로젝트 입니다.
+ 특정 집단에서 사용이 허가되는 문서 서식이 있다면 해당 서식을 정의하고, 만들어 져 있는 문서가 해당 서식에 적합한지를 판단합니다.
+ 본 프로젝트는 OWPML 모델을 이용하여 문서를 검증합니다.
+ DVC 모듈은 다음의 기능을 지원합니다.
+ 글자모양, 문단모양, 표, 특수문자, 테두리, 글머리표, 문단번호, 스타일, 하이퍼링크, 메크로
+ 추후 Hwpx 문서에서 지원하는 다양한 기능들도 추가로 업데이트 될 예정입니다.


## 개발 구성
예제는 아래 환경에서 개발 되었습니다.
+ Microsoft Windows 10 ![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
+ Microsoft Visual studio 2017 (v15.9.41) ![Visual Studio](https://img.shields.io/badge/Visual%20Studio-5C2D91.svg?style=for-the-badge&logo=visual-studio&logoColor=white)
+ C++


## OSS (Open Source Software)
프로젝트를 빌드하기 위해서는 아래 OSS(Open Source Software)가 필요합니다.
+ [Hancom OWPML Model](https://github.com/hancom-io/hwpx-owpml-model) // Apache 2.0 License
+ [jsoncpp](https://github.com/open-source-parsers/jsoncpp.git) // MIT License
  + jsoncpp 설치 순서
    + jsoncpp download
    + make windows VS2017 project (opensource\jsoncpp\cmake CMakeLists.txt -B json_git -G "Visual Studio 15 2017 Win64" 입력)
    + open jsoncpp.sln VS2017 project (opensource\jsoncpp\json_git\jsoncpp.sln)
    + build jsoncpp_lib project
    + confirm jsoncpp.dll & lib	
+ update_oss.bat 를 실행하면 기본적인 OSS 는 다운로드 됩니다.

jsoncpp를 처음 받게 되면 기본적으로 윈도우 빌드가 되어 있기 않기 때문에 관련 작업을 해 줘야 합니다.
윈도우용 프로젝트를 만들기 위해서는 cmake가 설치되어 있어야 하며  cmake는 
https://cmake.org/download/
에서 다운로드 하실수 있습니다.
윈도우(Windows)운영체제를 이용하고 있으므로 윈도우 버전을 설치할 필요가 있습니다.
Windows x64 Installer : cmake-3.24.2-windows-x86_64.msi

오픈소스 프로젝트는 아래 경로에 위치하여야 합니다.
  + ./opensource/hwpx-owpml-model  
  + ./opensource/jsoncpp


## Build
1. ./DVCModel.sln 솔루션 파일을 Visual studio 에서 불러옵니다.
2. 빌드
3. 빌드에 성공하면 다음과 같이 파일이 생성됩니다.
  + DVCModel.dll
  + DVCModel.lib 


## DLL 사용법
1. ./source/export/export.h 파일을 참조하시기 바랍니다.  
2. **`DVC::createDVC()`** 로 DVCModel 객체를 생성하세요.
3. **`IDVC::setCommand()`** 로 Diff 실행에 필요한 옵션을 설정하시기 바랍니다.
4. **`IDVC::doValidationCheck()`** 로 문서의 Validation 을 실행하시기 바랍니다.
5. **`IDVC::output()`** 로 Validation 결과를 출력합니다.
6. **`DVC::deleteHwpxDiff()`** 로 1번 에서 생성된 DVCModel 객체를 종료하여 주시기 바랍니다.

사용 예
```C++
#include "../../../export/export.h"


int wmain(int argc, wchar_t *argv[])
{
    bool printOutput = true;
    DVC::IDVC* dvc = DVC::createDVC();
    if (dvc)
    {        
        LPCWSTR msg = dvc->setCommand(argc, argv);
        if (msg != NULL)
            std::wcout << msg << std::endl;
        else
        {
            if (dvc->doValidationCheck() == true)
                dvc->output();
        }
    }
    DVC::deleteDVC(dvc);
}
```

## DVC Schema
1. jsonFullSpec.json 파일에 정의되어 있습니다.
2. 각 항목별 Check 하고 싶은 내용을 지정하고, 허용되는 내용을 기입하면 됩니다.
3. 모든 스팩을 다 사용해도 되지만 현 시점에서는 글자모양, 문단모양, 표, 특수문자, 테두리, 글머리표, 문단번호, 스타일, 하이퍼링크, 메크로 만 구현이 되어 있으며, 나머지는 추후 추가 업데이트 할 예정입니다.
4. jsonFullSpec.json에서 필요한 옵션만 가지고 와서 spec을 정의하면 됩니다.


## DVC Output Option (argument)

|짧은 옵션|긴 옵션|설명|
|---|---|---|
|== 출력 형식 정의 ==|
|-j|--format=json|출력 형식을 Json 형태로 표현한다. (Default value)|
|-x|--format=xml|출력 형식을 xml 형태로 표현한다.|
|== 결과 출력 ==|
|-c|--console|Console 화면에 출력 (Default value)|
|  |--file=[FilePath]|지정된 파일이름으로 출력 결과를 저장한다.|
|== 체크 수준 ==|
|-s|--simple|단일 체크. Validation 체크를 진행하다가 오류가 발생 시 그 즉시 체크를 멈추고 오류 알린다.|
|-a|--all|모든 Json 항목 체크 (Default value). Validation option에 정의되어 있는 모든 항목을 체크하고, 오류로 검출 된 모든 항목을 리포팅 한다.|
|== 출력 옵션 ==|
|-d|--default|기본 항목에 대한 출력 (Default Value)|
|-o|--alloption|모든 항목에 대한 출력|
|-t|--table|테이블 관련 출력 내용 - 테이블에 오류가 하나라도 나오면 그 부분만 체크하고 넘어감(테이블이 많은 문서의 경우, 퍼포먼스 관련하여 이점이 있음)|
|-i|--tabledetail|테이블 관련 출력 내용 - 테이블의 셀 단위로 체크하여 오류 보고|
|-p|--shape|도형 관련 출력 내용|
|-y|--style|스타일 관련 출력 내용|
|-k|--hyperlink|하이퍼링크 관련 출력 내용|
|== 공통 명령어 ==|
|-h, -H|--help|도움말|
|-v|--version|버전 정보|


## Demo
1. Examples\windows\ExampleWindows\ExampleWindows 프로젝트 실행 후 빌드 진행
2. 빌드 옵션에서 아래와 같이 옵션을 지정한다. 
**`-j --file=D:\Project\HWPXProjects\TestSample\DVCDebug\Result.json -s -t D:\Project\HWPXProjects\TestSample\DVCDebug\test.json "D:\Project\HWPXProjects\TestSample\DVCDebug\005_busan.hwpx`**  
Demo app 을 통해 HWPX 문서의 Validation Chek가 이루어지면 아래와 같은 결과를 확인하실 수 있습니다.  (Result.json)
```JSON
[
  {
    "charIDRef" : 6,
    "errorCode" : 1005,
    "isInTable" : false,
    "isInTableInTable" : false,
    "lineNo" : 4,
    "pageNo" : 2,
    "paraPrIDRef" : 0,
    "tableCol" : 0,
    "tableID" : 0,
    "tableRow" : 0,
    "text" : ""
  },
  {
    "charIDRef" : 6,
    "errorCode" : 1005,
    "isInTable" : false,
    "isInTableInTable" : false,
    "lineNo" : 6,
    "pageNo" : 2,
    "paraPrIDRef" : 2,
    "tableCol" : 0,
    "tableID" : 0,
    "tableRow" : 0,
    "text" : "레벨1"
  },

```
  + 보기 편하도록 띄어쓰기한 결과를 가져왔습니다.
    * errorCode 에 대한 정의는 "JsonModel.h" 파일에 정의되어 있습니다.

2. 추가 옵션은 **` --help`** 로 확인 하실 수 있습니다.

3. Demo 솔루션은  
  ./examples/windows/ExampleWindows.sln  
  에 구성되어 있습니다.


## License
자세한 내용은 [LICENSE](https://github.com/hancom-io/dvc/blob/main/LICENSE) 파일을 참고하세요

## Contact
문의사항은 [Discussions](https://github.com/hancom-io/dvc/discussions)로 자유롭게 문의 부탁합니다.
