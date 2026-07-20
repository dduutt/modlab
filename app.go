package main

import (
	"context"
	"errors"
	"fmt"
	"io"
	"sync"
	"time"

	"github.com/dduutt/modbus"
	"github.com/wailsapp/wails/v2/pkg/runtime"
	gserial "go.bug.st/serial"
	"go.bug.st/serial/enumerator"
)

type ServerInstance struct {
	cancel context.CancelFunc
	store  *modbus.MemoryDataStore
	conn   io.Closer
	handle io.Closer
}

type ClientInstance struct {
	transport modbus.Transport
	client    *modbus.Client
}

type App struct {
	ctx            context.Context
	clients        sync.Map
	servers        sync.Map
	clientsMu      sync.Mutex
	openSerialPort func(address string, mode *gserial.Mode) (io.ReadWriteCloser, error)
}

func NewApp() *App {
	return &App{}
}

// GetAvailablePorts scans the system for available serial ports
func (a *App) GetAvailablePorts() ([]string, error) {
	ports, err := enumerator.GetDetailedPortsList()
	if err != nil {
		return nil, err
	}
	var portNames []string
	for _, port := range ports {
		portNames = append(portNames, port.Name)
	}
	return portNames, nil
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) shutdown(ctx context.Context) {
	a.ClearAllConnections()
}

// ClearAllConnections forcefully stops all clients and servers. Useful for frontend hot-reloads.
func (a *App) ClearAllConnections() error {
	a.clientsMu.Lock()
	var closeErr error
	a.clients.Range(func(key, value interface{}) bool {
		ci := value.(*ClientInstance)
		if ci.transport != nil {
			closeErr = errors.Join(closeErr, ci.transport.Close())
		}
		a.clients.Delete(key)
		return true
	})
	a.clientsMu.Unlock()
	a.servers.Range(func(key, value interface{}) bool {
		si := value.(*ServerInstance)
		si.cancel()
		if si.handle != nil {
			si.handle.Close()
		}
		if si.conn != nil {
			si.conn.Close()
		}
		a.servers.Delete(key)
		return true
	})
	return closeErr
}

// ---------------------------------------------------------
// MASTER (CLIENT) LOGIC
// ---------------------------------------------------------

func getParity(p string) gserial.Parity {
	switch p {
	case "None", "N":
		return gserial.NoParity
	case "Even", "E":
		return gserial.EvenParity
	case "Odd", "O":
		return gserial.OddParity
	}
	return gserial.NoParity
}

func getStopBits(s uint) gserial.StopBits {
	switch s {
	case 1:
		return gserial.OneStopBit
	case 2:
		return gserial.TwoStopBits
	}
	return gserial.OneStopBit
}

func (a *App) openSerial(address string, mode *gserial.Mode) (io.ReadWriteCloser, error) {
	if a.openSerialPort != nil {
		return a.openSerialPort(address, mode)
	}
	return gserial.Open(address, mode)
}

func (a *App) disconnectMasterLocked(id string) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return nil
	}

	ci := val.(*ClientInstance)
	var closeErr error
	if ci.transport != nil {
		closeErr = ci.transport.Close()
	}
	a.clients.Delete(id)
	if closeErr != nil {
		return fmt.Errorf("close client %s: %w", id, closeErr)
	}
	return nil
}

// ConnectMaster creates and connects a Modbus Client
func (a *App) ConnectMaster(id string, protocol string, address string, rtuBaudRate uint, rtuDataBits uint, rtuParity string, rtuStopBits uint) error {
	a.clientsMu.Lock()
	defer a.clientsMu.Unlock()
	if err := a.disconnectMasterLocked(id); err != nil {
		return err
	}

	var transport modbus.Transport
	var client *modbus.Client

	if protocol == "tcp" {
		transport = modbus.NewTCPTransport(address, modbus.WithTCPTimeout(2*time.Second))
		client = modbus.NewClient(transport, modbus.WithTimeout(2*time.Second))
		if err := client.Connect(context.Background()); err != nil {
			transport.Close()
			return err
		}
	} else {
		mode := &gserial.Mode{
			BaudRate: int(rtuBaudRate),
			DataBits: int(rtuDataBits),
			Parity:   getParity(rtuParity),
			StopBits: getStopBits(rtuStopBits),
		}
		port, err := a.openSerial(address, mode)
		if err != nil {
			return err
		}
		transport = modbus.NewRTUTransport(port, modbus.WithRTUTimeout(2*time.Second))
		client = modbus.NewClient(transport, modbus.WithTimeout(2*time.Second))
	}

	a.clients.Store(id, &ClientInstance{
		transport: transport,
		client:    client,
	})
	return nil
}

// DisconnectMaster stops a Modbus Client
func (a *App) DisconnectMaster(id string) error {
	a.clientsMu.Lock()
	defer a.clientsMu.Unlock()
	return a.disconnectMasterLocked(id)
}

// ReadRegisters reads from the client
func (a *App) ReadRegisters(id string, unitId uint8, functionCode string, address uint16, count uint16) ([]uint16, error) {
	val, ok := a.clients.Load(id)
	if !ok {
		return nil, fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := ci.client.ForUnit(unitId)
	ctx := context.Background()

	switch functionCode {
	case "03":
		return client.ReadHoldingRegisters(ctx, address, count)
	case "04":
		return client.ReadInputRegisters(ctx, address, count)
	case "01":
		bools, err := client.ReadCoils(ctx, address, count)
		if err != nil {
			return nil, err
		}
		res := make([]uint16, len(bools))
		for i, b := range bools {
			if b {
				res[i] = 1
			}
		}
		return res, nil
	case "02":
		bools, err := client.ReadDiscreteInputs(ctx, address, count)
		if err != nil {
			return nil, err
		}
		res := make([]uint16, len(bools))
		for i, b := range bools {
			if b {
				res[i] = 1
			}
		}
		return res, nil
	}
	return nil, fmt.Errorf("unsupported function code")
}

// WriteRegister writes to the client
func (a *App) WriteRegister(id string, unitId uint8, address uint16, value uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := ci.client.ForUnit(unitId)
	ctx := context.Background()

	return client.WriteSingleRegister(ctx, address, value)
}

// WriteMultipleRegisters writes multiple registers to the client
func (a *App) WriteMultipleRegisters(id string, unitId uint8, address uint16, values []uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := ci.client.ForUnit(unitId)
	ctx := context.Background()

	return client.WriteMultipleRegisters(ctx, address, values)
}

// WriteCoil writes a single coil to the client
func (a *App) WriteCoil(id string, unitId uint8, address uint16, value uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := ci.client.ForUnit(unitId)
	ctx := context.Background()

	return client.WriteSingleCoil(ctx, address, value > 0)
}

// WriteMultipleCoils writes multiple coils to the client
func (a *App) WriteMultipleCoils(id string, unitId uint8, address uint16, values []uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := ci.client.ForUnit(unitId)
	ctx := context.Background()

	bools := make([]bool, len(values))
	for i, v := range values {
		bools[i] = (v > 0)
	}

	return client.WriteMultipleCoils(ctx, address, bools)
}

// ---------------------------------------------------------
// SLAVE (SERVER) LOGIC
// ---------------------------------------------------------

// WrappedDataStore intercepts writes to emit Wails events
type WrappedDataStore struct {
	store *modbus.MemoryDataStore
	ctx   context.Context
	id    string
}

func (w *WrappedDataStore) ReadCoils(address, quantity uint16) ([]bool, error) {
	return w.store.ReadCoils(address, quantity)
}

func (w *WrappedDataStore) ReadDiscreteInputs(address, quantity uint16) ([]bool, error) {
	return w.store.ReadDiscreteInputs(address, quantity)
}

func (w *WrappedDataStore) ReadHoldingRegisters(address, quantity uint16) ([]uint16, error) {
	return w.store.ReadHoldingRegisters(address, quantity)
}

func (w *WrappedDataStore) ReadInputRegisters(address, quantity uint16) ([]uint16, error) {
	return w.store.ReadInputRegisters(address, quantity)
}

func (w *WrappedDataStore) WriteCoils(address uint16, values []bool) error {
	err := w.store.WriteCoils(address, values)
	if err == nil {
		if w.ctx != nil {
			payload := make([]uint16, len(values))
			for i, v := range values {
				if v {
					payload[i] = 1
				}
			}
			runtime.EventsEmit(w.ctx, "slave_write", w.id, address, payload)
		}
	}
	return err
}

func (w *WrappedDataStore) WriteHoldingRegisters(address uint16, values []uint16) error {
	err := w.store.WriteHoldingRegisters(address, values)
	if err == nil {
		if w.ctx != nil {
			runtime.EventsEmit(w.ctx, "slave_write", w.id, address, values)
		}
	}
	return err
}

// StartSlave starts a Modbus Server
func (a *App) StartSlave(id string, protocol string, address string, rtuBaudRate uint, rtuDataBits uint, rtuParity string, rtuStopBits uint) error {
	a.StopSlave(id)

	store := modbus.NewMemoryDataStoreSized(65536, 65536, 65536, 65536)
	wstore := &WrappedDataStore{
		store: store,
		ctx:   a.ctx,
		id:    id,
	}

	handler := modbus.NewDataStoreHandler(wstore)

	ctx, cancel := context.WithCancel(context.Background())
	var conn io.ReadWriteCloser
	var handle io.Closer
	var err error

	if protocol == "tcp" {
		handle, err = modbus.StartTCPServer(ctx, address, handler)
		if err != nil {
			cancel()
			return err
		}
	} else {
		mode := &gserial.Mode{
			BaudRate: int(rtuBaudRate),
			DataBits: int(rtuDataBits),
			Parity:   getParity(rtuParity),
			StopBits: getStopBits(rtuStopBits),
		}
		port, err := gserial.Open(address, mode)
		if err != nil {
			cancel()
			return err
		}
		conn = port
		handle = modbus.StartRTUServer(ctx, port, handler)
	}

	a.servers.Store(id, &ServerInstance{
		cancel: cancel,
		store:  store,
		conn:   conn,
		handle: handle,
	})

	return nil
}

func (a *App) StopSlave(id string) error {
	if val, ok := a.servers.Load(id); ok {
		si := val.(*ServerInstance)
		si.cancel()
		if si.handle != nil {
			si.handle.Close()
		}
		if si.conn != nil {
			si.conn.Close()
		}
		a.servers.Delete(id)
	}
	return nil
}

// UpdateSlaveData allows the frontend to write data into the server memory manually
func (a *App) UpdateSlaveData(id string, address uint16, values []uint16, functionCode string) error {
	if val, ok := a.servers.Load(id); ok {
		si := val.(*ServerInstance)

		switch functionCode {
		case "01", "02":
			bools := make([]bool, len(values))
			for i, v := range values {
				bools[i] = (v > 0)
			}
			if functionCode == "01" {
				return si.store.WriteCoils(address, bools)
			}
			return si.store.WriteDiscreteInputs(address, bools)
		case "03":
			return si.store.WriteHoldingRegisters(address, values)
		case "04":
			return si.store.WriteInputRegisters(address, values)
		default:
			return fmt.Errorf("unsupported function code %q", functionCode)
		}
	}
	return fmt.Errorf("slave memory not found")
}

// GetSlaveData allows the frontend to fetch current memory
func (a *App) GetSlaveData(id string, address uint16, count uint16, functionCode string) ([]uint16, error) {
	if val, ok := a.servers.Load(id); ok {
		si := val.(*ServerInstance)
		switch functionCode {
		case "01", "02":
			var bools []bool
			var err error
			if functionCode == "01" {
				bools, err = si.store.ReadCoils(address, count)
			} else {
				bools, err = si.store.ReadDiscreteInputs(address, count)
			}
			if err != nil {
				return nil, err
			}
			res := make([]uint16, len(bools))
			for i, b := range bools {
				if b {
					res[i] = 1
				}
			}
			return res, nil
		case "03":
			return si.store.ReadHoldingRegisters(address, count)
		case "04":
			return si.store.ReadInputRegisters(address, count)
		default:
			return nil, fmt.Errorf("unsupported function code %q", functionCode)
		}
	}
	return nil, fmt.Errorf("slave memory not found")
}
