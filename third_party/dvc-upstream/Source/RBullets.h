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

namespace OWPMLReaderModule {

    typedef struct BulletHead {
        UINT            id = 0;
        std::wstring    bulletChar;
    };

    class RBullets
    {

    public:
        RBullets(OWPML::COwpmlDocumnet* document);
        ~RBullets();

    public:

        std::vector<BulletHead*>    getBullets() { return m_bullets; }
        BulletHead*                    getBullet(UINT id);

    private:        
        bool    readBullets(OWPML::COwpmlDocumnet* document);

        std::vector<BulletHead*>        m_bullets;

    };

}