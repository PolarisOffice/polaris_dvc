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


#include "OWPML\stdafx.h"
#include "OWPML\OWPML.h"
#include "OWPML\OwpmlForDec.h"
#include "OWPMLApi\OWPMLApi.h"
#include "OWPMLUtil\HncCtrlChDef.h"

class ReaderUtil
{
public:
    static OWPML::CObject* FindObject(OWPML::CObject* object, UINT refID);
    static OWPML::CObject* FindObjectFromParents(OWPML::CObject* object, UINT refID, bool findToFirst = false);    // findToFirst : 상위로 찾아가다가 마지막까지 찾았는데 Null이 나왔을 경우, 최상위로부터 다시 아래로 찾아나간다.
    static bool FindObjectAndSaveObj(OWPML::CObject* object, UINT refID, std::vector<OWPML::CObject*>& vecObj);

    static std::wstring s2ws(const std::string& str);
    static std::string ws2s(const std::wstring& wstr);
    static bool    findInVector(std::vector<UINT> obj, UINT value);
	static LPCWSTR ProcessingTextElement(OWPML::CT* pText);
};

