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

#include "pch.h"
#include "Util.h"

#ifdef OS_UNIX
#include <unistd.h>
#include <dirent.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <archive.h>
#include <archive_entry.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>
#else
#include <Windows.h>
#include <atlstr.h>
#include <stdio.h>
#include <io.h>
#endif // OS_UNIX
#include <sys/stat.h>
#include <locale>
#include <vector>
#include <cstring>

#include <codecvt>

#include "./Util.h"

std::string Util::wstring_to_string(const wchar_t* wstring) {
    // sizeof (char16_t) == (wchar_t) �϶�
#ifdef OS_UNIX
    return utf16_to_string(reinterpret_cast<const char16_t*>(wstring));
#else
    std::wstring wstrTemp(wstring);
    std::string strTemp;
    strTemp.assign(wstrTemp.begin(), wstrTemp.end());
    return strTemp;
#endif
}

std::u16string Util::wstring_to_utf16(const wchar_t* wstring) {
#ifdef OS_UNIX
    // sizeof (char16_t) == (wchar_t) 일때
    if (sizeof(char16_t) == sizeof(wchar_t)) {
        return reinterpret_cast<const char16_t*>(wstring);
    }
#else
    return Util::string_to_utf16(Util::wstring_to_string(wstring));
#endif
    return u"";
}

std::wstring Util::utf16_to_wstring(const std::u16string& utf16string)
{
#ifdef OS_UNIX
    // sizeof (char16_t) == (wchar_t) 일때
    if (sizeof(char16_t) == sizeof(wchar_t)) {
        return reinterpret_cast<const wchar_t*>(utf16string.c_str());
    }
#else
    std::wstring wstrTemp;
    std::string strTemp = Util::utf16_to_string(utf16string);
    wstrTemp.assign(strTemp.begin(), strTemp.end());

    return wstrTemp;
#endif

    return L"";
}

std::string Util::utf16_to_string(const std::u16string& utf16string) {
#ifdef OS_UNIX
    std::string utf8String;

    int size = 0;

    for (char16_t utf16char : utf16string) {
        char convertedUtf8[5] = { 0x00, };

        if (utf16char < 0x80) {
            convertedUtf8[0] = (utf16char >> 0 & 0x7F) | 0x00;
            size += 1;
        }
        else if (utf16char < 0x0800) {
            convertedUtf8[0] = (utf16char >> 6 & 0x1F) | 0xC0;
            convertedUtf8[1] = (utf16char >> 0 & 0x3F) | 0x80;
            size += 2;
        }
        else if (utf16char < 0x010000) {
            convertedUtf8[0] = (utf16char >> 12 & 0x0F) | 0xE0;
            convertedUtf8[1] = (utf16char >> 6 & 0x3F) | 0x80;
            convertedUtf8[2] = (utf16char >> 0 & 0x3F) | 0x80;
            size += 3;
        }
        else if (utf16char < 0x110000) {
            convertedUtf8[0] = (utf16char >> 18 & 0x07) | 0xF0;
            convertedUtf8[1] = (utf16char >> 12 & 0x3F) | 0x80;
            convertedUtf8[2] = (utf16char >> 6 & 0x3F) | 0x80;
            convertedUtf8[3] = (utf16char >> 0 & 0x3F) | 0x80;
            size += 4;
        }

        utf8String.append(convertedUtf8);
    }

    return utf8String;
#else
    CStringA utf8;
    int len = WideCharToMultiByte(CP_ACP, 0, (LPCWCH)utf16string.c_str(), -1, NULL, 0, 0, 0);
    if (len > 1)
    {
        char *ptr = utf8.GetBuffer(len - 1);
        if (ptr) WideCharToMultiByte(CP_ACP, 0, (LPCWCH)utf16string.c_str(), -1, ptr, len, 0, 0);
        utf8.ReleaseBuffer();
    }
    return std::string(utf8);
#endif
}

std::u16string Util::string_to_utf16(const std::string& str)
{
#ifdef OS_UNIX
    std::string strTemp = str;
#else
    std::string strTemp = std::string(CW2A(CA2W(str.c_str()), CP_UTF8));
#endif
    std::vector<unsigned long> unicode;
    size_t i = 0;
    while (i < strTemp.size())
    {
        unsigned long uni;
        size_t todo;
        // bool error = false;
        unsigned char ch = strTemp[i++];
        if (ch <= 0x7F)
        {
            uni = ch;
            todo = 0;
        }
        else if (ch <= 0xBF)
        {
            throw "not a UTF-8 string";
        }
        else if (ch <= 0xDF)
        {
            uni = ch & 0x1F;
            todo = 1;
        }
        else if (ch <= 0xEF)
        {
            uni = ch & 0x0F;
            todo = 2;
        }
        else if (ch <= 0xF7)
        {
            uni = ch & 0x07;
            todo = 3;
        }
        else
        {
            throw "not a UTF-8 string";
        }
        for (size_t j = 0; j < todo; ++j)
        {
            if (j == strTemp.size())
                throw "not a UTF-8 string";
            unsigned char ch = strTemp[i++];
            if (ch < 0x80 || ch > 0xBF)
                throw "not a UTF-8 string";
            uni <<= 6;
            uni += ch & 0x3F;
        }
        if (uni >= 0xD800 && uni <= 0xDFFF)
            throw "not a UTF-8 string";
        if (uni > 0x10FFFF)
            throw "not a UTF-8 string";
        unicode.push_back(uni);
    }

    std::u16string utf16;
    for (size_t i = 0; i < unicode.size(); ++i)
    {
        unsigned long uni = unicode[i];
        if (uni <= 0xFFFF)
        {
            utf16 += (char16_t)uni;
        }
        else
        {
            uni -= 0x10000;
            utf16 += (char16_t)((uni >> 10) + 0xD800);
            utf16 += (char16_t)((uni & 0x3FF) + 0xDC00);
        }
    }

    return utf16;
}

std::wstring Util::s2ws(const std::string& str)
{
    using convert_typeX = std::codecvt_utf8<wchar_t>;
    std::wstring_convert<convert_typeX, wchar_t> convertX;
    return convertX.from_bytes(str);
}

std::string Util::ws2s(const std::wstring& wstr)
{
    using convert_typeX = std::codecvt_utf8<wchar_t>;
    std::wstring_convert<convert_typeX, wchar_t> convertX;
    return convertX.to_bytes(wstr);
}

bool Util::findInVector(std::vector<UINT> obj, UINT value)
{
    std::vector<UINT>::iterator iter = obj.begin();

    for (iter; iter != obj.end(); iter++)
    {
        if ((*iter) == value)
            return true;
    }

    return false;
}

bool Util::getAbsolutePath(const DVC_STRING& argValue, DVC_STRING& returnPath)
{
    if (argValue.empty() == true) {
        return false;
    }

    // 존재여부 확인
    if (Util::isFileExist(argValue) == true) {
        returnPath.assign(argValue);
        return true;
    }

    // 현재 경로에 붙여보기
    DVC_STRING    filepath;

#ifdef OS_LINUX
    // Linux : ./ 로 시작한다면 현재 경로
    Error
        argValue.assign(argv[argIndex]);
#endif
#ifdef WINDOWS
    WCHAR currentDir[2048] = { 0x00, };
    GetCurrentDirectoryW(2047, currentDir);
    filepath.assign(currentDir);
    filepath.append(L"\\");
#endif
    filepath.append(argValue);

    if (Util::isFileExist(filepath) == true) {
        returnPath.assign(filepath);
        return true;
    }

    return false;
}

bool Util::isFileExist(const DVC_STRING& filePath)
{
#ifdef OS_LINUX
    if (access(filePath.c_str(), F_OK) < 0) {
#else
#define    F_OK    0        /* Test for existence.  */

    if (_waccess(filePath.c_str(), F_OK) < 0) {
#endif
        return false;
    }

    return true;
}

/* Polycode (https://github.com/ivansafrin/Polycode/) The function below is borrowed, and Polycode copyright and license contents are included. */
/*
Copyright (C) 2011 by Ivan Safrin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
*/

// Copyright (C) 2011 by Ivan Safrin
void Util::utf8toWStr(std::wstring& dest, const std::string& src) {
    dest.clear();
    wchar_t w = 0;
    int bytes = 0;
    wchar_t err = L'�';
    for (size_t i = 0; i < src.size(); i++) {
        unsigned char c = (unsigned char)src[i];
        if (c <= 0x7f) {//first byte
            if (bytes) {
                dest.push_back(err);
                bytes = 0;
            }
            dest.push_back((wchar_t)c);
        }
        else if (c <= 0xbf) {//second/third/etc byte
            if (bytes) {
                w = ((w << 6) | (c & 0x3f));
                bytes--;
                if (bytes == 0)
                    dest.push_back(w);
            }
            else
                dest.push_back(err);
        }
        else if (c <= 0xdf) {//2byte sequence start
            bytes = 1;
            w = c & 0x1f;
        }
        else if (c <= 0xef) {//3byte sequence start
            bytes = 2;
            w = c & 0x0f;
        }
        else if (c <= 0xf7) {//3byte sequence start
            bytes = 3;
            w = c & 0x07;
        }
        else {
            dest.push_back(err);
            bytes = 0;
        }
    }
    if (bytes)
        dest.push_back(err);
}

// Copyright (C) 2011 by Ivan Safrin
void Util::wstrToUtf8(std::string& dest, const std::wstring& src) {
    dest.clear();
    for (size_t i = 0; i < src.size(); i++) {
        wchar_t w = src[i];
        if (w <= 0x7f)
            dest.push_back((char)w);
        else if (w <= 0x7ff) {
            dest.push_back(0xc0 | ((w >> 6) & 0x1f));
            dest.push_back(0x80 | (w & 0x3f));
        }
        else if (w <= 0xffff) {
            dest.push_back(0xe0 | ((w >> 12) & 0x0f));
            dest.push_back(0x80 | ((w >> 6) & 0x3f));
            dest.push_back(0x80 | (w & 0x3f));
        }
        else if (w <= 0x10ffff) {
            dest.push_back(0xf0 | ((w >> 18) & 0x07));
            dest.push_back(0x80 | ((w >> 12) & 0x3f));
            dest.push_back(0x80 | ((w >> 6) & 0x3f));
            dest.push_back(0x80 | (w & 0x3f));
        }
        else
            dest.push_back('?');
    }
}

std::string Util::wstrToUtf8(const std::wstring& str) {
    std::string result;
    wstrToUtf8(result, str);
    return result;
}

std::wstring Util::utf8toWStr(const std::string& str) {
    std::wstring result;
    utf8toWStr(result, str);
    return result;
}


