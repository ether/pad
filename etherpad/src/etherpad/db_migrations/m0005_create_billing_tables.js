/**
 * Copyright 2009 Google Inc.
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *      http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS-IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import("etherpad.utils.isPrivateNetworkEdition");
import("sqlbase.sqlobj");

function run() {
  if (isPrivateNetworkEdition()) {
    return;
  }
  
  var idColspec = "SERIAL PRIMARY KEY";

  sqlobj.createTable('billing_purchase', {
    id: idColspec,
    type: "VARCHAR(64)", //"ENUM('onetimepurchase', 'subscription')",
    customer: "INT NOT NULL",
    product: "VARCHAR(128) NOT NULL",
    cost: "INT NOT NULL",
    coupon: "VARCHAR(128) NOT NULL",
    time: "timestamp",
    paidThrough: "timestamp",
    status: "VARCHAR(64)" // "ENUM('active', 'inactive')"
  }, {
    type: true,
    customer: true,
    product: true
  });
  
  sqlobj.createTable('billing_invoice', {
    id: idColspec,
    time: "timestamp",
    purchase: "INT NOT NULL",
    amt: "INT NOT NULL",
    status: "VARCHAR(64)" // ENUM('pending', 'paid', 'void', 'refunded')"
  }, {
    status: true
  });
  
  sqlobj.createTable('billing_transaction', {
    id: idColspec,
    customer: "INT",
    time: "timestamp",
    amt: "INT",
    payInfo: "VARCHAR(128)",
    txnId: "VARCHAR(128)", // depends on gateway used?
    status: "VARCHAR(64)" // ENUM('new', 'success', 'failure', 'pending')"
  }, {
    customer: true,
    txnId: true
  });
  
  sqlobj.createTable('billing_adjustment', {
    id: idColspec,
    transaction: "INT",
    invoice: "INT",
    time: "timestamp",
    amt: "INT"
  });
}
