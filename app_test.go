package main

import (
	"errors"
	"io"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/dduutt/modbus"
	gserial "go.bug.st/serial"
)

type countingSerial struct {
	mu             sync.Mutex
	closeCalls     int
	closeErr       error
	closeStarted   chan struct{}
	releaseClose   chan struct{}
	closeStartOnce sync.Once
}

func (s *countingSerial) Read([]byte) (int, error) {
	return 0, io.EOF
}

func (s *countingSerial) Write(p []byte) (int, error) {
	return len(p), nil
}

func (s *countingSerial) Close() error {
	s.mu.Lock()
	s.closeCalls++
	callNumber := s.closeCalls
	s.mu.Unlock()

	if callNumber == 1 && s.closeStarted != nil {
		s.closeStartOnce.Do(func() { close(s.closeStarted) })
	}
	if callNumber == 1 && s.releaseClose != nil {
		<-s.releaseClose
	}
	return s.closeErr
}

func (s *countingSerial) closeCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.closeCalls
}

func newTestClient(serial io.ReadWriteCloser) *ClientInstance {
	transport := modbus.NewRTUTransport(serial)
	return &ClientInstance{
		transport: transport,
		client:    modbus.NewClient(transport),
	}
}

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

func TestSlaveDataRejectsUnsupportedFunctionCode(t *testing.T) {
	app := NewApp()
	slaveID := "function-code-slave"
	port := "127.0.0.1:10504"

	if err := app.StartSlave(slaveID, "tcp", port, 9600, 8, "N", 1); err != nil {
		t.Fatalf("Failed to start slave: %v", err)
	}
	defer app.StopSlave(slaveID)

	if err := app.UpdateSlaveData(slaveID, 0, []uint16{1}, "99"); err == nil {
		t.Fatalf("Expected UpdateSlaveData to reject unsupported function code")
	}

	if _, err := app.GetSlaveData(slaveID, 0, 1, "99"); err == nil {
		t.Fatalf("Expected GetSlaveData to reject unsupported function code")
	}
}

func TestDisconnectMasterClosesRTUConnectionOnce(t *testing.T) {
	app := NewApp()
	serial := &countingSerial{}
	app.clients.Store("rtu", newTestClient(serial))

	if err := app.DisconnectMaster("rtu"); err != nil {
		t.Fatalf("DisconnectMaster failed: %v", err)
	}
	if got := serial.closeCount(); got != 1 {
		t.Fatalf("expected one serial close, got %d", got)
	}
	if _, ok := app.clients.Load("rtu"); ok {
		t.Fatal("expected client to be removed after disconnect")
	}

	if err := app.DisconnectMaster("rtu"); err != nil {
		t.Fatalf("repeated DisconnectMaster failed: %v", err)
	}
	if got := serial.closeCount(); got != 1 {
		t.Fatalf("expected repeated disconnect to keep one serial close, got %d", got)
	}
}

func TestDisconnectMasterReturnsSerialCloseError(t *testing.T) {
	closeErr := errors.New("serial close failed")
	app := NewApp()
	serial := &countingSerial{closeErr: closeErr}
	app.clients.Store("rtu", newTestClient(serial))

	err := app.DisconnectMaster("rtu")
	if !errors.Is(err, closeErr) {
		t.Fatalf("expected serial close error, got %v", err)
	}
	if _, ok := app.clients.Load("rtu"); ok {
		t.Fatal("expected client to be removed after close failure")
	}
}

func TestClearAllConnectionsClosesRTUConnectionOnce(t *testing.T) {
	app := NewApp()
	serial := &countingSerial{}
	app.clients.Store("rtu", newTestClient(serial))

	if err := app.ClearAllConnections(); err != nil {
		t.Fatalf("ClearAllConnections failed: %v", err)
	}
	if got := serial.closeCount(); got != 1 {
		t.Fatalf("expected cleanup to close serial once, got %d", got)
	}
	if _, ok := app.clients.Load("rtu"); ok {
		t.Fatal("expected client to be removed after cleanup")
	}
}

func TestConnectMasterClosesPreviousRTUConnectionOnce(t *testing.T) {
	app := NewApp()
	serials := []*countingSerial{{}, {}}
	openCalls := 0
	app.openSerialPort = func(_ string, _ *gserial.Mode) (io.ReadWriteCloser, error) {
		serial := serials[openCalls]
		openCalls++
		return serial, nil
	}

	if err := app.ConnectMaster("rtu", "rtu", "COM1", 9600, 8, "N", 1); err != nil {
		t.Fatalf("initial ConnectMaster failed: %v", err)
	}
	if err := app.ConnectMaster("rtu", "rtu", "COM1", 9600, 8, "N", 1); err != nil {
		t.Fatalf("replacement ConnectMaster failed: %v", err)
	}
	if got := serials[0].closeCount(); got != 1 {
		t.Fatalf("expected replaced serial to close once, got %d", got)
	}
	if got := serials[1].closeCount(); got != 0 {
		t.Fatalf("expected replacement serial to remain open, got %d closes", got)
	}

	if err := app.DisconnectMaster("rtu"); err != nil {
		t.Fatalf("final DisconnectMaster failed: %v", err)
	}
	if got := serials[1].closeCount(); got != 1 {
		t.Fatalf("expected active serial to close once, got %d", got)
	}
}

func TestConnectAndDisconnectMasterAreSerialized(t *testing.T) {
	app := NewApp()
	first := &countingSerial{}
	second := &countingSerial{}
	openStarted := make(chan struct{})
	releaseOpen := make(chan struct{})
	openCalls := 0
	app.openSerialPort = func(_ string, _ *gserial.Mode) (io.ReadWriteCloser, error) {
		openCalls++
		if openCalls == 2 {
			close(openStarted)
			<-releaseOpen
			return second, nil
		}
		return first, nil
	}

	if err := app.ConnectMaster("rtu", "rtu", "COM1", 9600, 8, "N", 1); err != nil {
		t.Fatalf("initial ConnectMaster failed: %v", err)
	}

	connectDone := make(chan error, 1)
	go func() {
		connectDone <- app.ConnectMaster("rtu", "rtu", "COM1", 9600, 8, "N", 1)
	}()
	<-openStarted

	disconnectDone := make(chan error, 1)
	go func() {
		disconnectDone <- app.DisconnectMaster("rtu")
	}()

	select {
	case err := <-disconnectDone:
		t.Fatalf("DisconnectMaster completed before replacement finished: %v", err)
	case <-time.After(50 * time.Millisecond):
	}

	close(releaseOpen)
	if err := <-connectDone; err != nil {
		t.Fatalf("replacement ConnectMaster failed: %v", err)
	}
	if err := <-disconnectDone; err != nil {
		t.Fatalf("DisconnectMaster failed: %v", err)
	}
	if _, ok := app.clients.Load("rtu"); ok {
		t.Fatal("expected serialized disconnect to remove replacement client")
	}
	if got := first.closeCount(); got != 1 {
		t.Fatalf("expected first serial to close once, got %d", got)
	}
	if got := second.closeCount(); got != 1 {
		t.Fatalf("expected replacement serial to close once, got %d", got)
	}
}
