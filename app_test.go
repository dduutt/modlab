package main

import (
	"net"
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
			if readData[i] != v {
				t.Errorf("Master coil read mismatch at offset %d: expected %v, got %v", i, v, readData[i])
			}
		}

		err = app.WriteCoil(masterID, 1, 21, 1)
		if err != nil {
			t.Fatalf("Failed to write coil from master: %v", err)
		}

		slaveCoils, err = app.GetSlaveData(slaveID, 20, 3, "01")
		if err != nil {
			t.Fatalf("Failed to get slave coils after master write: %v", err)
		}
		if slaveCoils[1] != 1 {
			t.Errorf("Master coil write mismatch at address 21: expected 1, got %v", slaveCoils[1])
		}
	})
}

func TestClearAllConnectionsReleasesServerPort(t *testing.T) {
	app := NewApp()

	slaveID := "cleanup-slave"
	port := "127.0.0.1:10503"

	if err := app.StartSlave(slaveID, "tcp", port, 9600, 8, "N", 1); err != nil {
		t.Fatalf("Failed to start slave: %v", err)
	}

	if err := app.ClearAllConnections(); err != nil {
		t.Fatalf("Failed to clear connections: %v", err)
	}

	time.Sleep(100 * time.Millisecond)

	if err := app.StartSlave(slaveID, "tcp", port, 9600, 8, "N", 1); err != nil {
		t.Fatalf("Failed to restart slave on same port after cleanup: %v", err)
	}
	defer app.StopSlave(slaveID)
}

func TestConnectMasterTCPRequiresOpenPort(t *testing.T) {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("Failed to reserve test port: %v", err)
	}
	port := ln.Addr().String()
	if err := ln.Close(); err != nil {
		t.Fatalf("Failed to close test listener: %v", err)
	}

	app := NewApp()
	if err := app.ConnectMaster("missing-server", "tcp", port, 9600, 8, "None", 1); err == nil {
		t.Fatalf("Expected ConnectMaster to fail when no TCP server is listening on %s", port)
	}

	if _, err := app.ReadRegisters("missing-server", 1, "03", 0, 1); err == nil {
		t.Fatalf("Expected missing-server client to remain disconnected after failed connect")
	}
}
