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

#pragma once

#include <iostream>
#include <string>
#include <vector>

#include "DVCDefine.h"

class Util
{
public:
    static std::string wstring_to_string(const wchar_t* wstring);
    static std::u16string wstring_to_utf16(const wchar_t* wstring);
    static std::string utf16_to_string(const std::u16string& utf16string);
    static std::u16string string_to_utf16(const std::string& str);
    static std::wstring utf16_to_wstring(const std::u16string& utf16string);

    static std::wstring s2ws(const std::string& str);
    static std::string ws2s(const std::wstring& wstr);    
    static bool    findInVector(std::vector<UINT> obj, UINT value);

    static bool isFileExist(const DVC_STRING& filePath);
    static bool getAbsolutePath(const DVC_STRING& argValue, DVC_STRING& returnPath);

    static void utf8toWStr(std::wstring& dest, const std::string& src);
    static void wstrToUtf8(std::string& dest, const std::wstring& src);
    static std::string wstrToUtf8(const std::wstring& str);
    static std::wstring utf8toWStr(const std::string& str);    
};

