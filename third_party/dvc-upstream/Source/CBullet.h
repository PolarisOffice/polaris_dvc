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
#include <string>

#include "DVCInterface.h"
#include "JsonModel.h"

namespace CheckListModule {
    class CBullet : public IBullet
    {
    public:
        CBullet(std::string dvcFile);
        CBullet(LPCSTR document);
        ~CBullet();

    public:
        DVCType getType() { return DVCType::Bullet; }
        std::map<std::string, int>    getMapData() { return m_mapCheckedList; }
        std::vector<UINT>        getCheckedList() { return m_checkedList; }

        BulletType                getBulletType();
        CheckType                getCheckType();
        std::vector<UINT>        getCodes();
        std::string                getBulletShapes();

    private:
        bool parsingDVCFile(std::string dvcFile);
        bool parsingDVCDocument(LPCSTR document);

        bool parsingElement(Json::Value root, void* Obj = 0);

        std::string                    m_dvcFile;
        std::vector<UINT>            m_checkedList;    // json 으로 설정되어 있는 항목들을 포함하고 있는 벡터. 이 값을 보고 실제 어떤 값들이 설정되어 있는지 확인한다.
        std::map<std::string, int>    m_mapCheckedList;

        BulletType                m_bulletType;
        CheckType                m_checkType;
        std::vector<UINT>        m_codes;
        std::string                m_bulletShapes;
    };
}
