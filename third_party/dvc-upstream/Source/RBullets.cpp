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
#include "RBullets.h"

#include "ReaderUtil.h"

namespace OWPMLReaderModule {

    RBullets::RBullets(OWPML::COwpmlDocumnet* document)
    {
        readBullets(document);
    }

    RBullets::~RBullets()
    {
        if (m_bullets.size() > 0)
        {
            std::vector<BulletHead*>::iterator iter = m_bullets.begin();

            for (iter; iter != m_bullets.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_bullets.clear();
        }
    }

    BulletHead* RBullets::getBullet(UINT id)
    {
        std::vector<BulletHead*>::iterator iter = m_bullets.begin();

        for (iter; iter != m_bullets.end(); iter++)
        {
            if ((*iter)->id == id)
                return (*iter);
        }

        return NULL;
    }

    bool RBullets::readBullets(OWPML::COwpmlDocumnet* document)
    {
        if (!document)
            return false;

        OWPML::CHWPMLHeadType* headType = document->GetHead();
        
        OWPML::CBullets* bullets = (OWPML::CBullets*)ReaderUtil::FindObject(headType, ID_HEAD_Bullets);

        if (bullets)
        {
            int nCnt = bullets->GetItemCnt();

            for (int i = 0; i < nCnt; i++)
            {
                OWPML::CBulletType* bulletType = bullets->Getbullet(i);
                
                if (bulletType)
                {
                    BulletHead* bullet = new BulletHead;

                    bullet->id = bulletType->GetId();
                    bullet->bulletChar = bulletType->GetChar();

                    m_bullets.push_back(bullet);
                }
            }
        }
        return true;
    }
}
