// PSRAM Enabled

#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include <Wire.h>
#include <M5EPD.h>

#define SERVICE_UUID        "45611d13-4fc8-4e04-a88e-02bb24054e22"
#define CHARACTERISTIC_UUID "04aaf1b3-5724-4cda-8f61-775585393a46"
#define KEYBOARD_REG 0x5c

BLECharacteristic * char_keyboard;
void setup_bluetooth_server() {
    BLEDevice::init("InkVT2");
    BLEServer * server = BLEDevice::createServer();
    BLEService * service = server->createService(SERVICE_UUID);
    char_keyboard = service->createCharacteristic(
                                           CHARACTERISTIC_UUID,
                                           BLECharacteristic::PROPERTY_READ |
                                           BLECharacteristic::PROPERTY_WRITE |
                                           BLECharacteristic::PROPERTY_NOTIFY |
                                           BLECharacteristic::PROPERTY_INDICATE
                                         );
    char_keyboard->setAccessPermissions(ESP_GATT_PERM_READ_ENCRYPTED | ESP_GATT_PERM_WRITE_ENCRYPTED);
    char_keyboard->setValue("e0");
    service->start();
    BLEAdvertising * advertising = BLEDevice::getAdvertising();
    advertising->addServiceUUID(SERVICE_UUID);
    advertising->setScanResponse(true);
    advertising->setMinPreferred(0x06);
    advertising->setMinPreferred(0x12);
    BLEDevice::startAdvertising();
    BLESecurity * security = new BLESecurity();
    security->setStaticPIN(123456);
    security->setAuthenticationMode(ESP_LE_AUTH_REQ_SC_MITM_BOND);
}

M5EPD_Canvas canvas(&M5.EPD);
int point[2][2];
void setup_m5paper() {
    M5.begin(true, true, true, true, true);
    // M5.EPD.SetRotation(90);
    // M5.TP.SetRotation(90);
    M5.EPD.Clear(true);
    M5.RTC.begin();
    M5.SHT30.Begin();
    canvas.createCanvas(960, 540);
    canvas.setTextSize(3);
    canvas.drawString("Hello World", 45, 350);
    canvas.pushCanvas(0, 0, UPDATE_MODE_DU4);
    Wire1.begin(/* SDA */ 25, /* SCL */ 32, (uint32_t) 400000U);
}

void setup() {
    setup_m5paper();
    setup_bluetooth_server();
    Serial.println("setup done");
}

void loop() {
    // cardkb
    Wire1.requestFrom(0x5F, 1);
    while (Wire1.available()) {
        char c = Wire1.read();
        if (c != 0) {
            Serial.println(c, HEX);
            std::string value = "kb";
            value[1] = c;
            char_keyboard->setValue(value);
            char_keyboard->notify(); // or indicate
        }
    }

    // touch
    if (M5.TP.avaliable()) {
        if (!M5.TP.isFingerUp()) {
            M5.TP.update();
            canvas.fillCanvas(0);
            bool is_update = false;
            for (int i = 0; i < 2; i++) {
                tp_finger_t FingerItem = M5.TP.readFinger(i);
                if ((point[i][0] != FingerItem.x) ||
                    (point[i][1] != FingerItem.y)) {
                    is_update   = true;
                    point[i][0] = FingerItem.x;
                    point[i][1] = FingerItem.y;
                    canvas.fillRect(FingerItem.x - 50, FingerItem.y - 50, 100,
                                    100, 15);
                    Serial.printf("Finger ID:%d-->X: %d*C  Y: %d  Size: %d\r\n",
                                  FingerItem.id, FingerItem.x, FingerItem.y,
                                  FingerItem.size);
                }
            }
            if (is_update) {
                canvas.pushCanvas(0, 0, UPDATE_MODE_DU4);
            }
        }
    }

    // temp
    M5.SHT30.UpdateData();
    float tem = M5.SHT30.GetTemperature();
    float hum = M5.SHT30.GetRelHumidity();
    //Serial.printf("Temperature: %2.2f*C  Humidity: %0.2f%%\r\n", tem, hum);
    // loop
    delay(100);
}
