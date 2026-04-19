/*
 * Copyright 2022 Hancom Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef __DVC_PRE_DEFINE__
#define __DVC_PRE_DEFINE__

#include <string>
#include <map>
#include <vector>

#ifdef OS_LINUX
#define DVC_STRING std::string
#define DVC_CHAR char

#ifndef DVC_LPCSTR
#define DVC_LPCSTR   const DVC_CHAR*
#endif

#define StrLiteral(STR_VALUE) STR_VALUE
#define StrLiteralA(STR_VALUE) STR_VALUE

#elif WINDOWS
#define DVC_STRING std::wstring
#define DVC_CHAR wchar_t

#ifndef DVC_LPCSTR
#define DVC_LPCSTR   const DVC_CHAR*
#endif

#define StrLiteral(STR_VALUE) L##STR_VALUE
#define StrLiteralA(STR_VALUE) STR_VALUE

#endif

namespace DVCString {
    // 아래 define 은 namespace 안에서 사용하기 위해 namespace 범위 안에서 사용하도록 제한
        // Common
    const static DVC_STRING DiffRemoveMark = StrLiteral("[-] : ");
    const static DVC_STRING DiffAddMark = StrLiteral("[+] : ");

    // Command parser error
    const static DVC_STRING CmdParserErrPlaceHolder = StrLiteral("%s");

    const static DVC_STRING Format = StrLiteral("format");
    const static DVC_STRING Json = StrLiteral("json");
    const static DVC_STRING Xml = StrLiteral("xml");
    const static DVC_STRING Hwpx = StrLiteral("hwpx");
    const static DVC_STRING Text = StrLiteral("text");
    const static DVC_STRING Console = StrLiteral("console");
    const static DVC_STRING File = StrLiteral("file");
    const static DVC_STRING Level = StrLiteral("level");
    const static DVC_STRING Simple = StrLiteral("simple");
    const static DVC_STRING All = StrLiteral("all");
    const static DVC_STRING Attribute = StrLiteral("attribute");
    const static DVC_STRING Help = StrLiteral("help");
    const static DVC_STRING Version = StrLiteral("version");

    const static DVC_STRING OutputDefault = StrLiteral("default");
    const static DVC_STRING OutputAllOption = StrLiteral("alloption");
    const static DVC_STRING OutputTable = StrLiteral("table");
    const static DVC_STRING OutputTableDetail = StrLiteral("tabledetail");
    const static DVC_STRING OutputShape = StrLiteral("shape");
    const static DVC_STRING OutputStyle = StrLiteral("style");
    const static DVC_STRING OutputHyperlink = StrLiteral("hyperlink");

    // ErrorInfos
    const static std::string CharIDRef = StrLiteralA("charIDRef");
    const static std::string ParaPrIDRef = StrLiteralA("paraPrIDRef");
    const static std::string errorText = StrLiteralA("text");
    const static std::string PageNo = StrLiteralA("pageNo");
    const static std::string LineNo = StrLiteralA("lineNo");
    const static std::string ErrorCode = StrLiteralA("errorCode");

    const static std::string TableID = StrLiteralA("tableID");
    const static std::string IsInTable = StrLiteralA("isInTable");
    const static std::string IsInTableInTable = StrLiteralA("isInTableInTable");
    const static std::string TableRow = StrLiteralA("tableRow");
    const static std::string TableCol = StrLiteralA("tableCol");
    const static std::string IsInShape = StrLiteralA("isInShape");
    const static std::string UseHyperlink = StrLiteralA("useHyperlink");
    const static std::string UseStyle = StrLiteralA("useStyle");
    const static std::string ErrorString = StrLiteralA("errorString");

    // Each OS
#ifdef OS_LINUX
    const static DVC_STRING PathSeparator = StrLiteral("/");
    const static DVC_STRING LineFeed = StrLiteral("\n");
#elif WINDOWS
    const static DVC_STRING PathSeparator = StrLiteral("\\");
    const static DVC_STRING LineFeed = StrLiteral("\r\n");
#endif
}

#endif    // __DVC_PRE_DEFINE__