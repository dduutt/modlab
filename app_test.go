package main

import (
	"testing"
	"time"
)

func TestMasterSlaveCommunication(t *testing.T) {
	app := NewApp()
	// Leave app.ctx as nil for tests to bypass Wails EventsEmit

	slaveID := "test-slave"
	masterID := "test-master"
	port := "127.0.0.1:10502"

	// 1. Start Slave
	err := app.StartSlave(slaveID, "tcp", port, 9600, 8, "N", 1)
	if err != nil {
		t.Fatalf("Failed to start slave: %v", err)
	}
	defer app.StopSlave(slaveID)

	// Wait for slave to listen
	time.Sleep(500 * time.Millisecond)

	// 2. Connect Master
	err = app.ConnectMaster(masterID, "tcp", port, 9600, 8, "None", 1)
	if err != nil {
		t.Fatalf("Failed to connect master: %v", err)
	}
	defer app.DisconnectMaster(masterID)

	// 3. Test Holding Registers (Read/Write)
	t.Run("HoldingRegisters", func(t *testing.T) {
		// Master writes multiple registers
		writeValues := []uint16{0x1234, 0x5678, 0x9ABC}
		err = app.WriteMultipleRegisters(masterID, 1, 10, writeValues)
		if err != nil {
			t.Fatalf("Failed to write multiple registers: %v", err)
		}

		// Verify Slave Memory
		slaveData, err := app.GetSlaveData(slaveID, 10, 3, "03")
		if err != nil {
			t.Fatalf("Failed to get slave data: %v", err)
		}
		for i, v := range writeValues {
			if slaveData[i] != v {
				t.Errorf("Holding Register mismatch at offset %d: expected %x, got %x", i, v, slaveData[i])
			}
		}

		// Master reads back
		readData, err := app.ReadRegisters(masterID, 1, "03", 10, 3)
		if err != nil {
			t.Fatalf("Failed to read registers from master: %v", err)
		}
		for i, v := range writeValues {
			if readData[i] != v {
				t.Errorf("Master read mismatch at offset %d: expected %x, got %x", i, v, readData[i])
			}
		}
	})

	// 4. Test Coils (Read/Write)
	t.Run("Coils", func(t *testing.T) {
		// Master writes a single coil to ON (0xFF00)
		// Our app.go doesn't have a specific `WriteCoil` function exposed yet for Master?
		// Wait, app.go `WriteRegister` only does Holding Register. We might need to manually update slave memory to test coil reading.
		
		err := app.UpdateSlaveData(slaveID, 20, []uint16{1, 0, 1}, "01")
		if err != nil {
			t.Fatalf("Failed to update slave coils: %v", err)
		}

		// Slave gets coils
		slaveCoils, err := app.GetSlaveData(slaveID, 20, 3, "01")
		if err != nil {
			t.Fatalf("Failed to get slave coils: %v", err)
		}
		expectedCoils := []uint16{1, 0, 1}
		for i, v := range expectedCoils {
			if slaveCoils[i] != v {
				t.Errorf("Coil mismatch at offset %d: expected %v, got %v", i, v, slaveCoils[i])
			}
		}

		// Master reads coils (01)
		readData, err := app.ReadRegisters(masterID, 1, "01", 20, 3)
		if err != nil {
			t.Fatalf("Failed to read coils from master: %v", err)
		}
		for i, v := range expectedCoils {
			// Master returns bools mapped to 1/0? Let's check how app.ReadRegisters handles "01"
			// Wait, app.go ReadRegisters handles "01" by returning []uint16{1 or 0}
			if readData[i] != v {
				t.Errorf("Master coil read mismatch at offset %d: expected %v, got %v", i, v, readData[i])
			}
		}
	})
}
