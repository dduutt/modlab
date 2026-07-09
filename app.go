package main

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/simonvetter/modbus"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type ServerMemory struct {
	Registers []uint16
	Coils     []bool
}

type App struct {
	ctx          context.Context
	clients      sync.Map
	servers      sync.Map
	serverMemory sync.Map
}

func NewApp() *App {
	return &App{}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) shutdown(ctx context.Context) {
	a.ClearAllConnections()
}

// ClearAllConnections forcefully stops all clients and servers. Useful for frontend hot-reloads.
func (a *App) ClearAllConnections() error {
	a.clients.Range(func(key, value interface{}) bool {
		client := value.(*modbus.ModbusClient)
		client.Close()
		a.clients.Delete(key)
		return true
	})
	a.servers.Range(func(key, value interface{}) bool {
		server := value.(*modbus.ModbusServer)
		server.Stop()
		a.servers.Delete(key)
		return true
	})
	a.serverMemory.Range(func(key, value interface{}) bool {
		a.serverMemory.Delete(key)
		return true
	})
	return nil
}

// ---------------------------------------------------------
// MASTER (CLIENT) LOGIC
// ---------------------------------------------------------

func getParity(p string) uint {
	switch p {
	case "None":
		return modbus.PARITY_NONE
	case "Even":
		return modbus.PARITY_EVEN
	case "Odd":
		return modbus.PARITY_ODD
	}
	return modbus.PARITY_NONE
}

// ConnectMaster creates and connects a Modbus Client
func (a *App) ConnectMaster(id string, protocol string, address string, rtuBaudRate uint, rtuDataBits uint, rtuParity string, rtuStopBits uint) error {
	// Disconnect existing if any
	if existing, ok := a.clients.Load(id); ok {
		existing.(*modbus.ModbusClient).Close()
		a.clients.Delete(id)
	}

	uri := ""
	if protocol == "tcp" {
		uri = "tcp://" + address
	} else {
		uri = "rtu://" + address
	}

	client, err := modbus.NewClient(&modbus.ClientConfiguration{
		URL:      uri,
		Speed:    rtuBaudRate,
		DataBits: rtuDataBits,
		Parity:   getParity(rtuParity),
		StopBits: rtuStopBits,
		Timeout:  2 * time.Second,
	})
	if err != nil {
		return err
	}

	err = client.Open()
	if err != nil {
		return err
	}

	a.clients.Store(id, client)
	return nil
}

// DisconnectMaster stops a Modbus Client
func (a *App) DisconnectMaster(id string) error {
	if val, ok := a.clients.Load(id); ok {
		val.(*modbus.ModbusClient).Close()
		a.clients.Delete(id)
	}
	return nil
}

// ReadRegisters reads from the client
func (a *App) ReadRegisters(id string, unitId uint8, functionCode string, address uint16, count uint16) ([]uint16, error) {
	val, ok := a.clients.Load(id)
	if !ok {
		return nil, fmt.Errorf("client %s not connected", id)
	}

	client := val.(*modbus.ModbusClient)
	client.SetUnitId(unitId)

	switch functionCode {
	case "03":
		return client.ReadRegisters(address, count, modbus.HOLDING_REGISTER)
	case "04":
		return client.ReadRegisters(address, count, modbus.INPUT_REGISTER)
	case "01":
		bools, err := client.ReadCoils(address, count)
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
		bools, err := client.ReadDiscreteInputs(address, count)
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

	client := val.(*modbus.ModbusClient)
	client.SetUnitId(unitId)

	return client.WriteRegister(address, value)
}

// WriteMultipleRegisters writes multiple registers to the client
func (a *App) WriteMultipleRegisters(id string, unitId uint8, address uint16, values []uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	client := val.(*modbus.ModbusClient)
	client.SetUnitId(unitId)

	return client.WriteRegisters(address, values)
}

// ---------------------------------------------------------
// SLAVE (SERVER) LOGIC
// ---------------------------------------------------------

type SlaveHandler struct {
	id  string
	app *App
}

func (h *SlaveHandler) HandleCoils(req *modbus.CoilsRequest) (res []bool, err error) {
	mem, ok := h.app.serverMemory.Load(h.id)
	if !ok {
		return nil, modbus.ErrIllegalDataAddress
	}
	m := mem.(*ServerMemory)

	if req.IsWrite {
		for i, val := range req.Args {
			m.Coils[int(req.Addr)+i] = val
		}
		return req.Args, nil
	} else {
		return m.Coils[req.Addr : req.Addr+req.Quantity], nil
	}
}

func (h *SlaveHandler) HandleDiscreteInputs(req *modbus.DiscreteInputsRequest) (res []bool, err error) {
	mem, ok := h.app.serverMemory.Load(h.id)
	if !ok {
		return nil, modbus.ErrIllegalDataAddress
	}
	m := mem.(*ServerMemory)
	return m.Coils[req.Addr : req.Addr+req.Quantity], nil
}

func (h *SlaveHandler) HandleHoldingRegisters(req *modbus.HoldingRegistersRequest) (res []uint16, err error) {
	mem, ok := h.app.serverMemory.Load(h.id)
	if !ok {
		return nil, modbus.ErrIllegalDataAddress
	}
	m := mem.(*ServerMemory)

	if req.IsWrite {
		for i, val := range req.Args {
			m.Registers[int(req.Addr)+i] = val
		}
		// Notify frontend
		runtime.EventsEmit(h.app.ctx, "slave_write", h.id, req.Addr, req.Args)
		return req.Args, nil
	} else {
		return m.Registers[req.Addr : req.Addr+req.Quantity], nil
	}
}

func (h *SlaveHandler) HandleInputRegisters(req *modbus.InputRegistersRequest) (res []uint16, err error) {
	mem, ok := h.app.serverMemory.Load(h.id)
	if !ok {
		return nil, modbus.ErrIllegalDataAddress
	}
	m := mem.(*ServerMemory)
	return m.Registers[req.Addr : req.Addr+req.Quantity], nil
}

// StartSlave starts a Modbus Server
func (a *App) StartSlave(id string, protocol string, address string) error {
	a.StopSlave(id)

	a.serverMemory.Store(id, &ServerMemory{
		Registers: make([]uint16, 65536),
		Coils:     make([]bool, 65536),
	})

	uri := ""
	if protocol == "tcp" {
		uri = "tcp://" + address
	} else {
		uri = "rtu://" + address
	}

	handler := &SlaveHandler{id: id, app: a}

	server, err := modbus.NewServer(&modbus.ServerConfiguration{
		URL:        uri,
		Timeout:    10 * time.Second,
		MaxClients: 5,
	}, handler)

	if err != nil {
		return err
	}

	err = server.Start()
	if err != nil {
		return err
	}

	a.servers.Store(id, server)
	return nil
}

func (a *App) StopSlave(id string) error {
	if val, ok := a.servers.Load(id); ok {
		val.(*modbus.ModbusServer).Stop()
		a.servers.Delete(id)
	}
	// We don't delete memory immediately so frontend can still see the last values
	return nil
}

// UpdateSlaveData allows the frontend to write data into the server memory manually
func (a *App) UpdateSlaveData(id string, address uint16, values []uint16) error {
	if val, ok := a.serverMemory.Load(id); ok {
		m := val.(*ServerMemory)
		for i, v := range values {
			if int(address)+i < len(m.Registers) {
				m.Registers[int(address)+i] = v
			}
		}
		return nil
	}
	return fmt.Errorf("slave memory not found")
}

// GetSlaveData allows the frontend to fetch current memory
func (a *App) GetSlaveData(id string, address uint16, count uint16) ([]uint16, error) {
	if val, ok := a.serverMemory.Load(id); ok {
		m := val.(*ServerMemory)
		if int(address+count) <= len(m.Registers) {
			return m.Registers[address : address+count], nil
		}
	}
	return nil, fmt.Errorf("slave memory not found or out of bounds")
}
