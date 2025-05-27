#include <Arduino.h>
#include <Fonts/FreeMonoBold9pt7b.h>
#include <GxEPD2_3C.h>
#include <GxEPD2_4C.h>
#include <GxEPD2_7C.h>
#include <GxEPD2_BW.h>
#include <WiFiMulti.h>
#include <HTTPClient.h>

// select the display class and display driver class in the following file (new
// style):
#include "GxEPD2_display_selection_new_style.h"

// or select the display constructor line in one of the following files (old
// style):
// #include "GxEPD2_display_selection.h"
// #include "GxEPD2_display_selection_added.h"

// alternately you can copy the constructor from GxEPD2_display_selection.h or
// GxEPD2_display_selection_added.h to here e.g. for Wemos D1 mini:
// GxEPD2_BW<GxEPD2_154_D67, GxEPD2_154_D67::HEIGHT>
// display(GxEPD2_154_D67(/*CS=D8*/ SS, /*DC=D3*/ 0, /*RST=D4*/ 2, /*BUSY=D2*/
// 4)); // GDEH0154D67

// for handling alternative SPI pins (ESP32, RP2040) see example
// GxEPD2_Example.ino

// void sayText();

WiFiMulti wifiMulti;

void setup() {

  Serial.begin(115200);

  Serial.println();
  Serial.println();
  Serial.println();

  for (uint8_t t = 4; t > 0; t--) {
    Serial.printf("[SETUP] WAIT %d...\n", t);
    Serial.flush();
    delay(1000);
  }

  wifiMulti.addAP("SSID", "PASSWD");

  // display.init(115200); // default 10ms reset pulse, e.g. for bare panels
  // with DESPI-C02
  display.init(115200, true, 2,
               false); // USE THIS for Waveshare boards with "clever" reset
                       // circuit, 2ms reset pulse
  // sayText();
  // display.hibernate();
}


void sayText(String dispText) {
  display.setRotation(2);
  display.setFont(&FreeMonoBold9pt7b);
  display.setTextColor(GxEPD_BLACK);
  int16_t tbx, tby;
  uint16_t tbw, tbh;
  display.getTextBounds(dispText, 0, 0, &tbx, &tby, &tbw, &tbh);
  // center the bounding box by transposition of the origin:
  uint16_t x, y = 0;
  // uint16_t x = ((display.width() - tbw) / 2) - tbx;
  // uint16_t y = ((display.height() - tbh) / 2) - tby;
  display.setFullWindow();
  display.firstPage();
  do {
    display.fillScreen(GxEPD_WHITE);
    display.setCursor(x, y);
    display.print(dispText);
  } while (display.nextPage());
}

void loop() {
  // wait for WiFi connection
  if ((wifiMulti.run() == WL_CONNECTED)) {

    HTTPClient http;

    Serial.print("[HTTP] begin...\n");
    // configure traged server and url
    // http.begin("https://www.howsmyssl.com/a/check", ca); //HTTPS
    http.begin("http://10.0.0.126:7272"); // HTTP

    Serial.print("[HTTP] GET...\n");
    // start connection and send HTTP header
    int httpCode = http.GET();

    // httpCode will be negative on error
    if (httpCode > 0) {
      // HTTP header has been send and Server response header has been handled
      Serial.printf("[HTTP] GET... code: %d\n", httpCode);

      // file found at server
      if (httpCode == HTTP_CODE_OK) {
        String payload = http.getString();
        Serial.println(payload);
        sayText(payload);
      }
    } else {
      Serial.printf("[HTTP] GET... failed, error: %s\n",
                        http.errorToString(httpCode).c_str());
    }

    http.end();
    if (httpCode == HTTP_CODE_OK) {
      delay(60000);
    }else{
      delay(10000);
    }
  }else{

    delay(2000);
  }

};